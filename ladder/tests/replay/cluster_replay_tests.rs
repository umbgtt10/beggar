// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The replay tier over the cluster simulation's logs: every instance's trace,
// written in the specification's JSON as its observer was told it and
// replayed from that instance's origin -- under any faults, restarts and
// cancellation policy, the reasons of its cancels included, every
// receipt rebuilt; with a line dropped, the gap found at its number and the
// replay resynchronised at the next checkpoint the log holds.

use crate::support::log_replay::LogReplay;
use crate::support::spec_writer::SpecWriter;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::scenario_trace_entry::ScenarioTraceEntry;
use beggar_core_validation::transport::fault_policy::FaultPolicy;
use beggar_core_validation::transport::message_kind::MessageKind;
use proptest::collection::vec;
use proptest::option::of;
use proptest::prelude::any;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use proptest::prelude::ProptestConfig;
use proptest::prelude::Strategy;
use serde_json::Value;

// A cluster run to quiescence under the policy, the workload, the restarts
// and the patience, then made to cancel what it still holds and run again
// until the abandons are delivered; its invariants checked before it starts
// and after every step -- which asks every node for its checkpoint, so every
// log opens with one and holds one at every step.
fn cluster(
    policy: FaultPolicy,
    nodes: u64,
    transfers: &[(bool, u64, u64, u64)],
    restarts: &[(u64, u64)],
    patience: Option<u64>,
) -> ClusterSimulation {
    let mut simulation = ClusterSimulation::new(nodes, policy, 2, 6, patience);
    for (pull, first, second, chunks) in transfers {
        let (first, second) = (first % nodes + 1, second % nodes + 1);
        if first == second {
            continue;
        }
        if *pull {
            simulation.pull(first, second, *chunks);
        } else {
            simulation.push(first, second, *chunks);
        }
    }
    simulation.check();
    let mut steps = 0;
    loop {
        for (_, node) in restarts.iter().filter(|(at, _)| *at == steps) {
            simulation.restart(node % nodes + 1);
        }
        if steps == 20_000 || !simulation.step() {
            break;
        }
        steps += 1;
        simulation.check();
    }
    simulation.cancel_abandoned();
    simulation.run_checked(20_000);
    simulation
}

// One instance's log: the instance, its lines, and which of them are
// checkpoints.
struct InstanceLog {
    instance: InstanceId,
    lines: Vec<Value>,
    checkpoints: Vec<bool>,
}

// Every instance's log from the nodes' traces, in the order its lines were
// written.
fn logs(traces: Vec<Vec<ScenarioTraceEntry>>) -> Vec<InstanceLog> {
    let mut logs: Vec<InstanceLog> = Vec::new();
    for entry in traces.into_iter().flatten() {
        let line = SpecWriter::step(&entry.command, &entry.receipt);
        let checkpoint = entry.command == Command::ShowRegistry;
        match logs.last_mut() {
            Some(log) if log.instance == entry.instance => {
                log.lines.push(line);
                log.checkpoints.push(checkpoint);
            }
            _ => logs.push(InstanceLog {
                instance: entry.instance,
                lines: vec![line],
                checkpoints: vec![checkpoint],
            }),
        }
    }
    logs
}

fn origin(instance: InstanceId) -> Origin {
    Origin::Fresh(Fresh {
        instance,
        retention: 0,
        bound: None,
    })
}

fn policy() -> impl Strategy<Value = FaultPolicy> {
    (any::<u64>(), vec(0u8..=30, 7), any::<bool>()).prop_map(|(seed, rates, run_kind)| {
        FaultPolicy {
            loss: rates[0],
            duplication: rates[1],
            delay: rates[2],
            reorder: rates[3],
            partition: rates[4],
            asymmetric: rates[5],
            selective: rates[6],
            selective_target: if run_kind {
                MessageKind::Run
            } else {
                MessageKind::Request
            },
            seed,
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn replay_of_an_instances_log_missing_a_line_finds_the_gap_and_resynchronises_at_the_next_checkpoint(
        policy in policy(),
        nodes in 3u64..=5,
        transfers in vec((any::<bool>(), 0u64..5, 0u64..5, 1u64..=8), 1..8),
        restarts in vec((0u64..60, 0u64..5), 0..3),
        patience in of(1u64..=20),
        which in any::<usize>(),
        dropped in any::<usize>(),
    ) {
        // Arrange
        let logs = logs(cluster(policy, nodes, &transfers, &restarts, patience).traces());
        let log = &logs[which % logs.len()];
        let dropped = dropped % log.lines.len();
        let checkpoint = (dropped + 1..log.lines.len()).find(|index| log.checkpoints[*index]);
        let mut lines = log.lines.clone();
        lines.remove(dropped);

        // Act
        let report = LogReplay::new(origin(log.instance)).replay(&lines);

        // Assert
        let number = dropped as u64 + 1;
        let after = checkpoint.map_or(0, |checkpoint| log.lines.len() - 1 - checkpoint);
        prop_assert_eq!(report.lines, log.lines.len() as u64 - 1);
        if dropped + 1 < log.lines.len() {
            prop_assert_eq!(report.gaps, vec![number..=number]);
        } else {
            prop_assert!(report.gaps.is_empty());
        }
        prop_assert_eq!(report.reconstructed, (dropped + after) as u64);
        prop_assert!(report.departures.is_empty(), "{:?}", report.departures);
    }

    #[test]
    fn replay_of_every_instances_log_from_any_cluster_rebuilds_every_receipt(
        policy in policy(),
        nodes in 3u64..=5,
        transfers in vec((any::<bool>(), 0u64..5, 0u64..5, 1u64..=8), 1..8),
        restarts in vec((0u64..60, 0u64..5), 0..3),
        patience in of(1u64..=20),
    ) {
        // Arrange
        let logs = logs(cluster(policy, nodes, &transfers, &restarts, patience).traces());

        // Act
        let reports: Vec<_> = logs
            .iter()
            .map(|log| LogReplay::new(origin(log.instance)).replay(&log.lines))
            .collect();

        // Assert
        prop_assert!(!logs.is_empty());
        for (log, report) in logs.iter().zip(&reports) {
            prop_assert_eq!(report.reconstructed, log.lines.len() as u64);
            prop_assert!(report.gaps.is_empty());
            prop_assert!(report.departures.is_empty(), "{:?}", report.departures);
        }
    }
}
