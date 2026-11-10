// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::command::Command;
use beggar::instance_id::InstanceId;
use beggar::observer::Observer;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::session_id::SessionId;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::scenario_harness::ScenarioHarness;
use beggar_core_validation::scenario_node::ScenarioNode;
use beggar_core_validation::trace_observer::TraceObserver;
use beggar_core_validation::transport::fault_policy::FaultPolicy;
use beggar_core_validation::transport::message_kind::MessageKind;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn cancel_abandoned_has_every_node_cancel_what_it_still_holds() {
    // Arrange
    let mut simulation = ClusterSimulation::new(
        2,
        FaultPolicy {
            selective: 100,
            selective_target: MessageKind::Run,
            ..FaultPolicy::default()
        },
        2,
        6,
        None,
    );
    simulation.push(1, 2, 4);
    simulation.run_checked(100);
    simulation.pull(1, 2, 4);
    simulation.run_checked(100);

    // Act
    simulation.cancel_abandoned();

    // Assert
    let violations = simulation.check();
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation
        .checkpoints()
        .iter()
        .all(|checkpoint| checkpoint.open.is_empty()));
}

#[test]
fn checkpoint_of_every_node_of_a_new_harness_holds_nothing_at_its_first_incarnation() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[1, 2, 3]);

    // Act
    let checkpoints = [
        harness.checkpoint(1),
        harness.checkpoint(2),
        harness.checkpoint(3),
    ];

    // Assert
    for (node, checkpoint) in (1..=3).zip(checkpoints) {
        assert_eq!(
            checkpoint.instance,
            InstanceId {
                node,
                incarnation: 1
            }
        );
        assert!(checkpoint.open.is_empty());
    }
}

#[test]
fn observe_on_a_trace_observer_records_the_call_under_its_instance() {
    // Arrange
    let trace = Rc::new(RefCell::new(Vec::new()));
    let instance = InstanceId {
        node: 3,
        incarnation: 2,
    };
    let mut observer = TraceObserver::new(instance, Rc::clone(&trace));
    let receipt = Receipt {
        number: 1,
        output: Output::RecoveryNotStarted,
    };

    // Act
    observer.observe(&Command::ShowRegistry, &receipt);

    // Assert
    let trace = trace.borrow();
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].instance, instance);
    assert_eq!(trace[0].command, Command::ShowRegistry);
    assert_eq!(trace[0].receipt, receipt);
}

#[test]
fn process_on_a_node_records_every_call_in_its_trace() {
    // Arrange
    let mut node = ScenarioNode::new(1);

    // Act
    let started = node.process(Command::StartRecovery {
        chunks: 4,
        sender: 1,
        receiver: 2,
    });

    // Assert
    let trace = node.trace();
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].instance, node.instance());
    assert_eq!(trace[0].receipt, started);
}

#[test]
fn restart_of_a_node_replaces_its_instance_by_an_empty_one_and_keeps_its_trace() {
    // Arrange
    let mut node = ScenarioNode::new(1);
    node.process(Command::StartRecovery {
        chunks: 4,
        sender: 1,
        receiver: 2,
    });

    // Act
    node.restart();

    // Assert
    let checkpoint = node.checkpoint();
    assert_eq!(
        checkpoint.instance,
        InstanceId {
            node: 1,
            incarnation: 2
        }
    );
    assert!(checkpoint.open.is_empty());
    let trace = node.trace();
    assert_eq!(trace.len(), 2);
    assert_eq!(trace[0].instance.incarnation, 1);
    assert_eq!(trace[1].instance.incarnation, 2);
}

#[test]
fn run_checked_of_a_pull_answers_with_what_the_server_holds_and_completes_it() {
    // Arrange
    let mut simulation = ClusterSimulation::new(3, FaultPolicy::default(), 2, 4, None);
    simulation.pull(1, 2, 6);

    // Act
    let violations = simulation.run_checked(100);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfers = simulation.transfers();
    assert_eq!(transfers.len(), 1);
    assert_eq!(
        (
            transfers[0].sender,
            transfers[0].receiver,
            transfers[0].chunks
        ),
        (2, 1, 4)
    );
    assert!(transfers[0].sender_completed);
    assert_eq!(transfers[0].receiver_completions, 1);
}

#[test]
fn run_checked_of_a_push_completes_it_and_leaves_every_node_holding_nothing() {
    // Arrange
    let mut simulation = ClusterSimulation::new(3, FaultPolicy::default(), 2, 8, None);
    simulation.push(1, 2, 5);

    // Act
    let violations = simulation.run_checked(100);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfers = simulation.transfers();
    assert_eq!(transfers.len(), 1);
    assert!(transfers[0].sender_completed);
    assert_eq!(transfers[0].receiver_completions, 1);
    assert!(simulation
        .checkpoints()
        .iter()
        .all(|checkpoint| checkpoint.open.is_empty()));
}

#[test]
fn run_checked_past_its_limit_reports_the_cluster_unsettled() {
    // Arrange
    let mut simulation = ClusterSimulation::new(3, FaultPolicy::default(), 1, 8, None);
    simulation.push(1, 2, 8);

    // Act
    let violations = simulation.run_checked(3);

    // Assert
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("not settled"), "{violations:#?}");
}

#[test]
fn run_checked_with_an_impatient_receiver_has_its_sender_cancel_the_pages_it_still_holds() {
    // Arrange
    let mut simulation = ClusterSimulation::new(
        2,
        FaultPolicy {
            delay: 100,
            ..FaultPolicy::default()
        },
        1,
        6,
        Some(1),
    );
    simulation.push(1, 2, 6);

    // Act
    let violations = simulation.run_checked(200);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfer = &simulation.transfers()[0];
    assert!(transfer.sender_cancelled, "{transfer:?}");
    assert!(!transfer.sender_completed, "{transfer:?}");
}

#[test]
fn start_through_the_harness_is_answered_by_the_sender_alone() {
    // Arrange
    let mut harness = ScenarioHarness::new(&[1, 2]);

    // Act
    let receipt = harness.start(1, 2, 4);

    // Assert
    assert_eq!(
        receipt.output,
        Output::RecoveryStarted {
            session_id: SessionId {
                node: 1,
                incarnation: 1,
                count: 1
            }
        }
    );
    assert_eq!(harness.checkpoint(1).open.len(), 1);
    assert!(harness.checkpoint(2).open.is_empty());
}

#[test]
fn step_of_a_simulation_with_nothing_to_do_is_false() {
    // Arrange
    let mut simulation = ClusterSimulation::new(3, FaultPolicy::default(), 2, 8, None);

    // Act
    let stepped = simulation.step();

    // Assert
    assert!(!stepped);
}

#[test]
fn step_with_patience_has_a_receiver_cancel_the_session_that_stalls() {
    // Arrange
    let mut simulation = ClusterSimulation::new(
        2,
        FaultPolicy {
            delay: 100,
            ..FaultPolicy::default()
        },
        1,
        6,
        Some(1),
    );
    simulation.push(1, 2, 3);

    // Act
    let violations = simulation.run_checked(100);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfer = &simulation.transfers()[0];
    assert!(transfer.cancellations > 0, "{transfer:?}");
}

#[test]
fn traces_of_a_simulation_hold_each_nodes_calls() {
    // Arrange
    let mut simulation = ClusterSimulation::new(2, FaultPolicy::default(), 4, 8, None);
    simulation.push(1, 2, 4);

    // Act
    simulation.run_checked(100);

    // Assert
    let traces = simulation.traces();
    assert_eq!(traces.len(), 2);
    assert!(matches!(
        traces[0][0].command,
        Command::StartRecovery { .. }
    ));
    assert!(traces[1]
        .iter()
        .any(|entry| matches!(entry.command, Command::AdoptRecovery { .. })));
}
