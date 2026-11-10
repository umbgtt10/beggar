// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::session_id::SessionId;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::scenario_harness::ScenarioHarness;
use beggar_core_validation::scenario_reason::ScenarioReason;
use beggar_core_validation::transport::fault_policy::FaultPolicy;

const A: u64 = 1;
const B: u64 = 2;

// B started a session of six chunks to A and sent its first half; A adopted it
// and received that half.
fn half_sent() -> (ScenarioHarness, SessionId) {
    let mut harness = ScenarioHarness::new(&[A, B]);
    let session = named(harness.start(B, A, 6));
    harness.send(B, session, 1, 3, A);
    harness.adopt(A, session, 6, B);
    harness.receive(A, session, 1, 3, B);
    (harness, session)
}

fn named(receipt: Receipt<ScenarioReason>) -> SessionId {
    match receipt.output {
        Output::RecoveryStarted { session_id } => session_id,
        other => panic!("not started: {other:?}"),
    }
}

#[test]
fn restart_leaves_the_node_holding_nothing_under_its_next_incarnation() {
    // Arrange
    let (mut harness, _) = half_sent();

    // Act
    harness.restart(B);

    // Assert
    let checkpoint = harness.checkpoint(B);
    assert_eq!(
        checkpoint.instance,
        InstanceId {
            node: B,
            incarnation: 2
        }
    );
    assert!(checkpoint.open.is_empty());
}

#[test]
fn restart_of_a_receiver_adopts_a_later_run_again_under_its_senders_name() {
    // Arrange
    let (mut harness, session) = half_sent();
    harness.restart(A);

    // Act
    let adopted = harness.adopt(A, session, 6, B);
    let received = harness.receive(A, session, 4, 6, B);

    // Assert
    assert_eq!(
        adopted.output,
        Output::RecoveryStarted {
            session_id: session
        }
    );
    assert_eq!(
        received.output,
        Output::RecoveryAdvanced { held: 3, chunks: 6 }
    );
}

#[test]
fn restart_of_a_sender_leaves_its_old_session_not_started_there() {
    // Arrange
    let (mut harness, session) = half_sent();
    harness.restart(B);

    // Act
    let sent = harness.send(B, session, 4, 6, A);

    // Assert
    assert_eq!(sent.output, Output::RecoveryNotStarted);
}

#[test]
fn restart_of_a_sender_leaves_its_receivers_session_open_as_it_stood() {
    // Arrange
    let (mut harness, session) = half_sent();

    // Act
    harness.restart(B);

    // Assert
    let open = harness.checkpoint(A).open;
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].session_id, session);
    assert_eq!(open[0].chunk_set.ids(), vec![1, 2, 3]);
}

#[test]
fn restart_of_a_sender_names_its_next_sessions_under_its_next_incarnation() {
    // Arrange
    let (mut harness, first) = half_sent();
    harness.restart(B);

    // Act
    let next = named(harness.start(B, A, 2));

    // Assert
    assert_eq!(
        next,
        SessionId {
            node: B,
            incarnation: 2,
            count: 1
        }
    );
    assert_ne!(next, first);
}

#[test]
fn restart_of_nodes_mid_transfer_keeps_every_invariant() {
    // Arrange
    let mut simulation = ClusterSimulation::new(3, FaultPolicy::default(), 1, 6);
    simulation.push(1, 2, 6);
    simulation.push(3, 1, 6);
    simulation.pull(2, 3, 6);
    for _ in 0..6 {
        simulation.step();
    }

    // Act
    simulation.restart(1);
    simulation.restart(2);
    let violations = simulation.run_checked(1_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation
        .transfers()
        .iter()
        .any(|transfer| transfer.interrupted));
}
