// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar_core_validation::cluster_invariants::ClusterInvariants;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::scenario_harness::ScenarioHarness;
use beggar_core_validation::scenario_reason::ScenarioReason;
use beggar_core_validation::transport::fault_policy::FaultPolicy;

const A: u64 = 1;
const B: u64 = 2;
const C: u64 = 3;

fn named(receipt: Receipt<ScenarioReason>) -> SessionId {
    match receipt.output {
        Output::RecoveryStarted { session_id } => session_id,
        other => panic!("not started: {other:?}"),
    }
}

#[test]
fn pull_answered_completes_at_both_ends() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[A, B]);
    let session = named(harness.start(B, A, 6));

    // Act
    let sent = [
        harness.send(B, session, 1, 3, A),
        harness.send(B, session, 4, 6, A),
    ];
    let adopted = harness.adopt(A, session, 6, B);
    let received = [
        harness.receive(A, session, 1, 3, B),
        harness.receive(A, session, 4, 6, B),
    ];

    // Assert
    assert_eq!(sent[1].output, Output::RecoveryCompleted);
    assert_eq!(
        adopted.output,
        Output::RecoveryStarted {
            session_id: session
        }
    );
    assert_eq!(
        received.map(|receipt| receipt.output),
        [
            Output::RecoveryAdvanced { held: 3, chunks: 6 },
            Output::RecoveryCompleted
        ]
    );
    assert!(harness.checkpoint(A).open.is_empty());
    assert!(harness.checkpoint(B).open.is_empty());
}

#[test]
fn push_completes_at_both_ends_whatever_the_order_its_runs_arrive_in() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[A, B]);
    let session = named(harness.start(B, A, 6));
    harness.send(B, session, 1, 3, A);
    harness.send(B, session, 4, 6, A);

    // Act
    harness.adopt(A, session, 6, B);
    let later_run = harness.receive(A, session, 4, 6, B);
    let earlier_run = harness.receive(A, session, 1, 3, B);

    // Assert
    assert_eq!(
        later_run.output,
        Output::RecoveryAdvanced { held: 3, chunks: 6 }
    );
    assert_eq!(earlier_run.output, Output::RecoveryCompleted);
    assert!(harness.checkpoint(A).open.is_empty());
    assert!(harness.checkpoint(B).open.is_empty());
}

#[test]
fn run_checked_of_a_lossless_cluster_completes_every_transfer_pulled_or_pushed() {
    // Arrange
    let mut simulation = ClusterSimulation::new(
        4,
        FaultPolicy {
            duplication: 20,
            delay: 20,
            reorder: 30,
            seed: 11,
            ..FaultPolicy::default()
        },
        2,
        5,
        None,
        0,
    );
    simulation.pull(1, 2, 8);
    simulation.push(3, 1, 7);
    simulation.pull(4, 3, 3);
    simulation.push(2, 4, 6);

    // Act
    let violations = simulation.run_checked(1_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert_eq!(simulation.transfers().len(), 4);
    let invariants = ClusterInvariants::new(simulation.transfers(), simulation.checkpoints());
    let unfinished = invariants.liveness();
    assert!(unfinished.is_empty(), "{unfinished:#?}");
}

#[test]
fn send_and_receive_on_a_node_doing_both_keeps_each_session_on_its_side() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[A, B, C]);
    let outgoing = named(harness.start(A, C, 2));
    let incoming = named(harness.start(B, A, 2));
    harness.adopt(A, incoming, 2, B);
    harness.adopt(C, outgoing, 2, A);

    // Act
    let refused = [
        harness
            .process(
                A,
                Command::ReceiveRecoveryPartial {
                    session_id: outgoing,
                    first: 1,
                    last: 2,
                    sender: A,
                    receiver: C,
                },
            )
            .output,
        harness
            .process(
                A,
                Command::SendRecoveryPartial {
                    session_id: incoming,
                    first: 1,
                    last: 2,
                    sender: B,
                    receiver: A,
                },
            )
            .output,
    ];
    let sent = harness.send(A, outgoing, 1, 2, C);
    let received = harness.receive(A, incoming, 1, 2, B);

    // Assert
    assert_eq!(
        refused,
        [
            Output::RecoveryRejected(Rejection::WrongSide),
            Output::RecoveryRejected(Rejection::WrongSide)
        ]
    );
    assert_eq!(sent.output, Output::RecoveryCompleted);
    assert_eq!(received.output, Output::RecoveryCompleted);
    assert_eq!(harness.checkpoint(A).open.len(), 0);
    assert_eq!(harness.checkpoint(C).open.len(), 1);
}

#[test]
fn start_for_a_short_answer_opens_a_smaller_session_that_completes() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[A, B]);
    let held_by_b = 5;

    // Act
    let session = named(harness.start(B, A, held_by_b));
    harness.send(B, session, 1, held_by_b, A);
    harness.adopt(A, session, held_by_b, B);
    let received = harness.receive(A, session, 1, held_by_b, B);

    // Assert
    assert_eq!(received.output, Output::RecoveryCompleted);
    assert!(harness.checkpoint(A).open.is_empty());
}

#[test]
fn start_on_a_node_serving_two_keeps_a_session_for_each_under_names_of_its_own() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[A, B, C]);

    // Act
    let to_a = named(harness.start(B, A, 3));
    let to_c = named(harness.start(B, C, 2));
    harness.adopt(A, to_a, 3, B);
    harness.adopt(C, to_c, 2, B);
    harness.send(B, to_c, 1, 2, C);
    let c_done = harness.receive(C, to_c, 1, 2, B);

    // Assert
    assert_ne!(to_a, to_c);
    assert_eq!(c_done.output, Output::RecoveryCompleted);
    assert_eq!(harness.checkpoint(B).open.len(), 1);
    assert_eq!(harness.checkpoint(A).open.len(), 1);
    assert!(harness.checkpoint(C).open.is_empty());
}
