// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::output::Output;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::scenario_reason::ScenarioReason;
use beggar_core_validation::scenario_trace_entry::ScenarioTraceEntry;
use beggar_core_validation::transport::fault_policy::FaultPolicy;

const LIMIT: u64 = 10_000;

// Whether some receiver answers that it lacks a chunk.
fn lacking(answers: &[Output<ScenarioReason>]) -> bool {
    answers.iter().any(
        |answer| matches!(answer, Output::MissingChunksListed { missing } if !missing.is_empty()),
    )
}

// Six transfers among four nodes over a transport that loses, duplicates and
// reorders, pulled and pushed in pages of two, at the retention given.
fn lossy(retention: u64, patience: Option<u64>) -> ClusterSimulation {
    let policy = FaultPolicy {
        loss: 25,
        duplication: 10,
        reorder: 20,
        seed: 5,
        ..FaultPolicy::default()
    };
    let mut simulation = ClusterSimulation::new(4, policy, 2, 6, patience, retention);
    simulation.push(1, 2, 6);
    simulation.pull(3, 1, 6);
    simulation.push(2, 4, 5);
    simulation.pull(4, 2, 4);
    simulation.push(3, 4, 6);
    simulation.pull(1, 3, 6);
    simulation
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

#[test]
fn cancel_abandoned_at_retention_0_leaves_no_receiver_a_session_to_answer_for() {
    // Arrange
    let mut simulation = lossy(0, None);
    let quiet = simulation.run_checked(LIMIT);
    let lacked = simulation.missing_chunks();

    // Act
    simulation.cancel_abandoned();
    let told = simulation.run_checked(LIMIT);

    // Assert
    assert!(quiet.is_empty(), "{quiet:#?}");
    assert!(told.is_empty(), "{told:#?}");
    assert!(lacking(&lacked), "no receiver lacked a chunk: {lacked:?}");
    assert!(simulation
        .missing_chunks()
        .iter()
        .all(|answer| *answer == Output::RecoveryNotStarted));
}

#[test]
fn cancel_abandoned_at_the_highest_retention_keeps_what_each_receiver_lacked() {
    // Arrange
    let mut simulation = lossy(u64::MAX, None);
    let quiet = simulation.run_checked(LIMIT);
    let lacked = simulation.missing_chunks();

    // Act
    simulation.cancel_abandoned();
    let told = simulation.run_checked(LIMIT);

    // Assert
    assert!(quiet.is_empty(), "{quiet:#?}");
    assert!(told.is_empty(), "{told:#?}");
    assert!(lacking(&lacked), "no receiver lacked a chunk: {lacked:?}");
    assert_eq!(simulation.missing_chunks(), lacked);
}

#[test]
fn check_quiescent_over_a_lossy_transport_finds_each_receiver_lacking_what_never_reached_it() {
    // Arrange
    let mut simulation = lossy(0, None);
    while simulation.step() {}

    // Act
    let violations = simulation.check_quiescent();

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let answers = simulation.missing_chunks();
    assert!(lacking(&answers), "no receiver lacks a chunk: {answers:?}");
}

#[test]
fn run_checked_at_the_highest_retention_answers_every_call_but_a_query_as_at_retention_0() {
    // Arrange
    let mut keeping_none = lossy(0, Some(6));
    let mut keeping_all = lossy(u64::MAX, Some(6));

    // Act
    let violations = [
        keeping_none.run_checked(LIMIT),
        keeping_all.run_checked(LIMIT),
    ];

    // Assert
    assert!(violations.iter().all(Vec::is_empty), "{violations:#?}");
    assert_eq!(
        transitions(keeping_none.traces()),
        transitions(keeping_all.traces())
    );
    let kept: usize = keeping_all
        .checkpoints()
        .iter()
        .map(|checkpoint| checkpoint.retained.len())
        .sum();
    assert!(kept > 0, "nothing was retained");
    assert_ne!(keeping_none.missing_chunks(), keeping_all.missing_chunks());
}
