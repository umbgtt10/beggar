// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar_core_validation::cluster_invariants::ClusterInvariants;
use beggar_core_validation::cluster_simulation::ClusterSimulation;
use beggar_core_validation::transport::fault_policy::FaultPolicy;
use beggar_core_validation::transport::message_kind::MessageKind;

// Six transfers among four nodes, pulled and pushed, in pages of two.
fn loaded(policy: FaultPolicy) -> ClusterSimulation {
    let mut simulation = ClusterSimulation::new(4, policy, 2, 6, None);
    simulation.push(1, 2, 6);
    simulation.pull(3, 1, 6);
    simulation.push(2, 4, 5);
    simulation.pull(4, 2, 4);
    simulation.push(3, 4, 6);
    simulation.pull(1, 3, 6);
    simulation
}

fn unfinished(simulation: &mut ClusterSimulation) -> Vec<String> {
    ClusterInvariants::new(simulation.transfers(), simulation.checkpoints()).liveness()
}

#[test]
fn cancel_abandoned_after_loss_ends_every_abandoned_session_and_settles_the_cluster() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        loss: 25,
        seed: 5,
        ..FaultPolicy::default()
    });
    let abandoned = simulation.run_checked(10_000);

    // Act
    simulation.cancel_abandoned();
    let told = simulation.run_checked(10_000);

    // Assert
    assert!(abandoned.is_empty(), "{abandoned:#?}");
    assert!(told.is_empty(), "{told:#?}");
    let invariants = ClusterInvariants::new(simulation.transfers(), simulation.checkpoints());
    let violations = invariants.violations();
    assert!(violations.is_empty(), "{violations:#?}");
    let open = invariants.settled();
    assert!(open.is_empty(), "{open:#?}");
    assert!(simulation
        .transfers()
        .iter()
        .any(|transfer| transfer.cancellations > 0));
}

#[test]
fn run_checked_across_a_partition_loses_what_crossed_it_and_completes_what_follows_the_heal() {
    // Arrange
    let mut simulation = ClusterSimulation::new(
        2,
        FaultPolicy {
            partition: 100,
            ..FaultPolicy::default()
        },
        2,
        6,
        None,
    );
    simulation.push(1, 2, 4);
    let cut = simulation.run_checked(100);

    // Act
    simulation.heal();
    simulation.push(1, 2, 4);
    let healed = simulation.run_checked(100);

    // Assert
    assert!(cut.is_empty(), "{cut:#?}");
    assert!(healed.is_empty(), "{healed:#?}");
    let transfers = simulation.transfers();
    assert!(transfers[0].delivered.is_empty());
    assert_eq!(transfers[1].receiver_completions, 1);
    assert!(simulation.counts().partitioned > 0);
}

#[test]
fn run_checked_over_asymmetric_cuts_keeps_every_invariant() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        asymmetric: 50,
        seed: 7,
        ..FaultPolicy::default()
    });

    // Act
    let violations = simulation.run_checked(10_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation.counts().asymmetric > 0);
}

#[test]
fn run_checked_under_delay_completes_every_transfer() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        delay: 50,
        seed: 4,
        ..FaultPolicy::default()
    });

    // Act
    let violations = simulation.run_checked(10_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation.counts().delayed > 0);
    let unfinished = unfinished(&mut simulation);
    assert!(unfinished.is_empty(), "{unfinished:#?}");
}

#[test]
fn run_checked_under_duplication_never_counts_a_chunk_twice_and_completes_every_transfer() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        duplication: 50,
        seed: 2,
        ..FaultPolicy::default()
    });

    // Act
    let violations = simulation.run_checked(10_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation.counts().duplicated > 0);
    let unfinished = unfinished(&mut simulation);
    assert!(unfinished.is_empty(), "{unfinished:#?}");
}

#[test]
fn run_checked_under_full_duplication_reopens_a_completed_session_once_by_its_late_copy() {
    // Arrange
    let mut simulation = ClusterSimulation::new(
        2,
        FaultPolicy {
            duplication: 100,
            ..FaultPolicy::default()
        },
        2,
        6,
        None,
    );
    simulation.push(1, 2, 4);

    // Act
    let violations = simulation.run_checked(100);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfer = &simulation.transfers()[0];
    assert_eq!(transfer.receiver_completions, 1);
    assert_eq!(transfer.reopens, 1);
    assert_eq!(transfer.late_copies, 1);
    assert_eq!(simulation.checkpoints()[1].open.len(), 1);
}

#[test]
fn run_checked_under_loss_abandons_exactly_the_sessions_a_chunk_never_reached() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        loss: 25,
        seed: 5,
        ..FaultPolicy::default()
    });

    // Act
    let violations = simulation.run_checked(10_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation.counts().lost > 0);
    let abandoned = simulation
        .checkpoints()
        .iter()
        .map(|checkpoint| checkpoint.open.len())
        .sum::<usize>();
    assert!(abandoned > 0, "no session was abandoned");
}

#[test]
fn run_checked_under_reordering_completes_every_transfer() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        reorder: 50,
        seed: 3,
        ..FaultPolicy::default()
    });

    // Act
    let violations = simulation.run_checked(10_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(simulation.counts().reordered > 0);
    let unfinished = unfinished(&mut simulation);
    assert!(unfinished.is_empty(), "{unfinished:#?}");
}

#[test]
fn run_checked_under_selective_loss_of_runs_answers_every_pull_and_delivers_no_chunk() {
    // Arrange
    let mut simulation = loaded(FaultPolicy {
        selective: 100,
        selective_target: MessageKind::Run,
        ..FaultPolicy::default()
    });

    // Act
    let violations = simulation.run_checked(10_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfers = simulation.transfers();
    assert_eq!(transfers.len(), 6);
    assert!(transfers
        .iter()
        .all(|transfer| transfer.delivered.is_empty()));
    assert!(simulation.counts().selective > 0);
}

#[test]
fn run_checked_with_impatient_receivers_under_delay_reopens_cancelled_sessions_by_their_late_runs()
{
    // Arrange
    let mut simulation = ClusterSimulation::new(
        3,
        FaultPolicy {
            delay: 100,
            ..FaultPolicy::default()
        },
        1,
        6,
        Some(1),
    );
    simulation.push(1, 2, 6);
    simulation.push(3, 2, 6);

    // Act
    let violations = simulation.run_checked(1_000);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    let transfers = simulation.transfers();
    assert!(transfers.iter().any(|transfer| transfer.cancellations > 0));
    assert!(transfers.iter().any(|transfer| transfer.reopens > 0));
}
