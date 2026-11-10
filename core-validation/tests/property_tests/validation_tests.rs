// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The cluster simulation over random clusters, workloads, fault mixes and
// retentions: every invariant after every delivery, abandonment exact and
// each receiver lacking what has not reached it at quiescence, every
// abandoned session then ended by the cancellation policy, every call but a
// query answered alike at retention 0 and at the top of u64, every transfer
// completed at both ends when nothing was lost -- a pull's request duplicated
// being answered twice, as asking again would be -- and no fault counted that
// was not configured. Each case is its seed: a failure replays.

use beggar::command::Command;
use beggar_core_validation::cluster_invariants::ClusterInvariants;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::scenario_trace_entry::ScenarioTraceEntry;
use beggar_core_validation::transport::fault_policy::FaultPolicy;
use beggar_core_validation::transport::message_kind::MessageKind;
use proptest::collection::vec;
use proptest::option::of;
use proptest::prelude::any;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::prop_oneof;
use proptest::prelude::proptest;
use proptest::prelude::Just;
use proptest::prelude::ProptestConfig;
use proptest::prelude::Strategy;

const LIMIT: u64 = 20_000;

// The cluster stepped to quiescence or the limit, restarting each node at the
// step given, checked after every delivery and at quiescence; then the
// policy's last word, and the cluster run again to quiescence, checked: how
// many steps the first run took, and the violations of both.
fn driven(
    simulation: ClusterSimulation,
    nodes: u64,
    restarts: &[(u64, u64)],
) -> (ClusterSimulation, u64, Vec<String>) {
    let mut simulation = simulation;
    let mut violations = Vec::new();
    let mut steps = 0;
    loop {
        for (_, node) in restarts.iter().filter(|(at, _)| *at == steps) {
            simulation.restart(node % nodes + 1);
        }
        if steps == LIMIT || !simulation.step() {
            break;
        }
        steps += 1;
        violations.extend(simulation.check());
    }
    violations.extend(simulation.check_quiescent());
    simulation.cancel_abandoned();
    violations.extend(simulation.run_checked(LIMIT));
    (simulation, steps, violations)
}

// Pulls and pushes among the nodes, a pair of the same node skipped.
fn load(
    simulation: &mut ClusterSimulation,
    nodes: u64,
    transfers: &[(bool, u64, u64, u64)],
) -> usize {
    let mut made = 0;
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
        made += 1;
    }
    made
}

// A fault policy of seven rates, its selective fault aimed at runs or at
// requests.
fn policy(seed: u64, rates: &[u8], run_kind: bool) -> FaultPolicy {
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
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn step_of_any_cluster_under_any_faults_keeps_every_invariant_after_every_delivery(
        seed in any::<u64>(),
        nodes in 3u64..=5,
        rates in vec(0u8..=30, 7),
        run_kind in any::<bool>(),
        page in 1u64..=4,
        holding in 1u64..=8,
        transfers in transfers(),
        restarts in vec((0u64..60, 0u64..5), 0..3),
        patience in of(1u64..=20),
        retention in prop_oneof![Just(0u64), 1u64..=3, Just(u64::MAX)],
    ) {
        // Arrange
        let policy = policy(seed, &rates, run_kind);
        let mut simulation = ClusterSimulation::new(nodes, policy, page, holding, patience, retention);
        load(&mut simulation, nodes, &transfers);

        // Act
        let (mut simulation, steps, violations) = driven(simulation, nodes, &restarts);
        let open = ClusterInvariants::new(simulation.transfers(), simulation.checkpoints()).settled();

        // Assert
        prop_assert!(steps < LIMIT, "not settled after {steps} steps, seed {seed}");
        prop_assert!(violations.is_empty(), "seed {seed}: {violations:#?}");
        prop_assert!(open.is_empty(), "seed {seed}: {open:#?}");
        let counts = simulation.counts();
        prop_assert!(policy.loss > 0 || counts.lost == 0);
        prop_assert!(policy.duplication > 0 || counts.duplicated == 0);
        prop_assert!(policy.delay > 0 || counts.delayed == 0);
        prop_assert!(policy.reorder > 0 || counts.reordered == 0);
        prop_assert!(policy.partition > 0 || counts.partitioned == 0);
        prop_assert!(policy.asymmetric > 0 || counts.asymmetric == 0);
        prop_assert!(policy.selective > 0 || counts.selective == 0);
    }

    #[test]
    fn step_of_any_cluster_at_the_highest_retention_answers_every_call_but_a_query_as_at_retention_0(
        seed in any::<u64>(),
        nodes in 3u64..=5,
        rates in vec(0u8..=30, 7),
        run_kind in any::<bool>(),
        page in 1u64..=4,
        holding in 1u64..=8,
        transfers in transfers(),
        restarts in vec((0u64..60, 0u64..5), 0..3),
        patience in of(1u64..=20),
    ) {
        // Arrange
        let mut keeping_none =
            ClusterSimulation::new(nodes, policy(seed, &rates, run_kind), page, holding, patience, 0);
        let mut keeping_all =
            ClusterSimulation::new(nodes, policy(seed, &rates, run_kind), page, holding, patience, u64::MAX);
        load(&mut keeping_none, nodes, &transfers);
        load(&mut keeping_all, nodes, &transfers);

        // Act
        let (keeping_none, _, none_violations) = driven(keeping_none, nodes, &restarts);
        let (keeping_all, _, all_violations) = driven(keeping_all, nodes, &restarts);

        // Assert
        prop_assert!(none_violations.is_empty(), "seed {seed}: {none_violations:#?}");
        prop_assert!(all_violations.is_empty(), "seed {seed}: {all_violations:#?}");
        prop_assert_eq!(transitions(keeping_none.traces()), transitions(keeping_all.traces()));
    }

    #[test]
    fn run_checked_of_any_lossless_cluster_completes_every_transfer_at_both_ends(
        seed in any::<u64>(),
        nodes in 3u64..=5,
        rates in vec(0u8..=40, 3),
        page in 1u64..=4,
        holding in 1u64..=8,
        transfers in transfers(),
    ) {
        // Arrange
        let policy = FaultPolicy {
            duplication: rates[0],
            delay: rates[1],
            reorder: rates[2],
            seed,
            ..FaultPolicy::default()
        };
        let mut simulation = ClusterSimulation::new(nodes, policy, page, holding, None, 0);
        let made = load(&mut simulation, nodes, &transfers);

        // Act
        let violations = simulation.run_checked(LIMIT);

        // Assert
        prop_assert!(violations.is_empty(), "seed {seed}: {violations:#?}");
        let transfers = simulation.transfers().len() as u64;
        prop_assert!(transfers >= made as u64, "seed {seed}: {transfers} transfers of {made} made");
        prop_assert!(
            transfers - made as u64 <= simulation.counts().duplicated,
            "seed {seed}: more answers than requests duplicated"
        );
        let unfinished =
            ClusterInvariants::new(simulation.transfers(), simulation.checkpoints()).liveness();
        prop_assert!(unfinished.is_empty(), "seed {seed}: {unfinished:#?}");
        simulation.cancel_abandoned();
        let told = simulation.run_checked(LIMIT);
        prop_assert!(told.is_empty(), "seed {seed}: {told:#?}");
        let open = ClusterInvariants::new(simulation.transfers(), simulation.checkpoints()).settled();
        prop_assert!(open.is_empty(), "seed {seed}: {open:#?}");
    }
}

fn transfers() -> impl Strategy<Value = Vec<(bool, u64, u64, u64)>> {
    vec((any::<bool>(), 0u64..5, 0u64..5, 1u64..=8), 1..8)
}

// Every call of every node's trace but a query.
fn transitions(traces: Vec<Vec<ScenarioTraceEntry>>) -> Vec<ScenarioTraceEntry> {
    traces
        .into_iter()
        .flatten()
        .filter(|entry| {
            !matches!(
                entry.command,
                Command::ListRecoveries
                    | Command::ShowRecovery { .. }
                    | Command::ListMissingChunks { .. }
                    | Command::ShowRegistry
            )
        })
        .collect()
}
