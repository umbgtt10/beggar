// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::checkpoint::Checkpoint;
use beggar::chunk_set::ChunkSet;
use beggar::instance_id::InstanceId;
use beggar::output::Output;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;
use beggar_core_validation::cluster_invariants::ClusterInvariants;
use beggar_core_validation::transfer::Transfer;
use std::collections::BTreeSet;
use std::ops::RangeInclusive;

const SESSION: SessionId = SessionId {
    node: 2,
    incarnation: 1,
    count: 1,
};

// Each node's checkpoint: node 1 receiving the session, holding the chunks
// given; nodes 2 and 3 holding nothing.
fn cluster(received: &[u64]) -> Vec<Checkpoint> {
    vec![
        node(1, vec![record(4, received)]),
        node(2, Vec::new()),
        node(3, Vec::new()),
    ]
}

// A node's checkpoint at its first incarnation, its name count the sessions
// it started: node 2 started the one transfer, the others none.
fn node(node: u64, open: Vec<SessionRecord>) -> Checkpoint {
    Checkpoint {
        instance: InstanceId {
            node,
            incarnation: 1,
        },
        retention: 0,
        bound: None,
        open,
        retained: Vec::new(),
        name_count: u64::from(node == SESSION.node),
        next_receipt: 9,
    }
}

fn record(chunks: u64, ids: &[u64]) -> SessionRecord {
    let mut chunk_set = ChunkSet::new();
    for id in ids {
        chunk_set.add(*id, *id);
    }
    SessionRecord {
        session_id: SESSION,
        chunks,
        sender: 2,
        receiver: 1,
        chunk_set,
    }
}

// Runs of chunk ids from their first and last.
fn runs(pairs: &[(u64, u64)]) -> Vec<RangeInclusive<u64>> {
    pairs.iter().map(|(first, last)| *first..=*last).collect()
}

// A transfer from node 2 to node 1 of four chunks, every one sent, the first
// two delivered and held by its receiver.
fn transfer() -> Transfer {
    Transfer {
        session_id: SESSION,
        sender: 2,
        receiver: 1,
        chunks: 4,
        sent: BTreeSet::from([1, 2, 3, 4]),
        delivered: BTreeSet::from([1, 2]),
        held: BTreeSet::from([1, 2]),
        receiver_open: true,
        sender_completed: true,
        receiver_completions: 0,
        reopens: 0,
        late_copies: 0,
        unsound: 0,
        interrupted: false,
        cancellations: 0,
        last_delivery: 0,
        sender_cancelled: false,
    }
}

#[test]
fn abandonment_excepts_a_transfer_the_policy_cancelled() {
    // Arrange
    let mut cancelled = transfer();
    cancelled.receiver_open = false;
    cancelled.cancellations = 1;
    let invariants = ClusterInvariants::new(
        vec![cancelled],
        vec![node(1, Vec::new()), node(2, Vec::new())],
    );

    // Act
    let violations = invariants.abandonment();

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn abandonment_names_a_receiver_holding_nothing_with_a_chunk_missing() {
    // Arrange
    let mut closed = transfer();
    closed.receiver_open = false;
    let invariants =
        ClusterInvariants::new(vec![closed], vec![node(1, Vec::new()), node(2, Vec::new())]);

    // Act
    let violations = invariants.abandonment();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn abandonment_names_a_receiver_still_open_with_every_chunk_delivered() {
    // Arrange
    let mut delivered = transfer();
    delivered.delivered = BTreeSet::from([1, 2, 3, 4]);
    let invariants = ClusterInvariants::new(vec![delivered], cluster(&[1, 2]));

    // Act
    let violations = invariants.abandonment();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn accounting_names_a_transfer_with_a_receipt_its_chunks_did_not_call_for() {
    // Arrange
    let mut unsound = transfer();
    unsound.unsound = 1;
    let invariants = ClusterInvariants::new(vec![unsound], cluster(&[1, 2]));

    // Act
    let violations = invariants.accounting();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn agreement_names_a_record_of_no_transfer_and_one_on_other_terms() {
    // Arrange
    let mut stranger = record(4, &[]);
    stranger.session_id.count = 9;
    let invariants = ClusterInvariants::new(
        vec![transfer()],
        vec![
            node(1, vec![record(5, &[1, 2]), stranger]),
            node(2, Vec::new()),
        ],
    );

    // Act
    let violations = invariants.agreement();

    // Assert
    assert_eq!(violations.len(), 2, "{violations:#?}");
}

#[test]
fn bystanders_name_a_record_held_by_a_node_outside_its_transfer() {
    // Arrange
    let mut checkpoints = cluster(&[1, 2]);
    checkpoints[2] = node(3, vec![record(4, &[])]);
    let invariants = ClusterInvariants::new(vec![transfer()], checkpoints);

    // Act
    let violations = invariants.bystanders();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn canonical_names_a_checkpoint_whose_records_are_out_of_name_order() {
    // Arrange
    let mut later = record(4, &[]);
    later.session_id.count = 2;
    let mut second = transfer();
    second.session_id = later.session_id;
    let invariants = ClusterInvariants::new(
        vec![transfer(), second],
        vec![
            node(1, vec![later, record(4, &[1, 2])]),
            node(2, Vec::new()),
        ],
    );

    // Act
    let violations = invariants.canonical();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn gaps_name_a_receiver_answering_for_a_session_it_does_not_hold_at_retention_0() {
    // Arrange
    let mut closed = transfer();
    closed.receiver_open = false;
    let invariants =
        ClusterInvariants::new(vec![closed], vec![node(1, Vec::new()), node(2, Vec::new())]);

    // Act
    let violations = invariants.gaps(&[Output::MissingChunksListed {
        missing: runs(&[(3, 4)]),
    }]);

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn gaps_name_a_receiver_answering_other_chunks_lacking_from_a_session_it_holds_open() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![transfer()], cluster(&[1, 2]));

    // Act
    let violations = invariants.gaps(&[Output::MissingChunksListed {
        missing: runs(&[(4, 4)]),
    }]);

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn gaps_of_a_receiver_answering_for_a_session_it_does_not_hold_above_retention_0_are_none() {
    // Arrange
    let mut closed = transfer();
    closed.receiver_open = false;
    let keeping = Checkpoint {
        retention: 3,
        ..node(1, Vec::new())
    };
    let invariants = ClusterInvariants::new(vec![closed], vec![keeping, node(2, Vec::new())]);

    // Act
    let violations = invariants.gaps(&[Output::MissingChunksListed {
        missing: runs(&[(3, 4)]),
    }]);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn gaps_of_receivers_answering_what_has_not_reached_them_are_none() {
    // Arrange
    let mut closed = transfer();
    closed.session_id = SessionId {
        count: 2,
        ..SESSION
    };
    closed.receiver_open = false;
    let invariants = ClusterInvariants::new(vec![transfer(), closed], cluster(&[1, 2]));

    // Act
    let violations = invariants.gaps(&[
        Output::MissingChunksListed {
            missing: runs(&[(3, 4)]),
        },
        Output::RecoveryNotStarted,
    ]);

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn liveness_names_a_transfer_its_receiver_never_completed() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![transfer()], cluster(&[1, 2]));

    // Act
    let violations = invariants.liveness();

    // Assert
    assert_eq!(violations.len(), 2, "{violations:#?}");
}

#[test]
fn names_minted_names_an_instance_whose_count_is_not_its_started_sessions() {
    // Arrange
    let mut checkpoints = cluster(&[1, 2]);
    checkpoints[0].name_count = 1;
    checkpoints[1].name_count = 2;
    let invariants = ClusterInvariants::new(vec![transfer()], checkpoints);

    // Act
    let violations = invariants.names_minted();

    // Assert
    assert_eq!(violations.len(), 2, "{violations:#?}");
}

#[test]
fn reopens_name_a_session_reopened_without_a_late_copy() {
    // Arrange
    let mut reopened = transfer();
    reopened.receiver_completions = 1;
    reopened.reopens = 1;
    let invariants = ClusterInvariants::new(vec![reopened], cluster(&[1, 2]));

    // Act
    let violations = invariants.reopens();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn settled_names_every_session_still_open() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![transfer()], cluster(&[1, 2]));

    // Act
    let violations = invariants.settled();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn subsets_name_a_receiver_holding_a_chunk_never_delivered() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![transfer()], cluster(&[1, 2, 3]));

    // Act
    let violations = invariants.subsets();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn subsets_name_a_sender_holding_a_chunk_never_sent() {
    // Arrange
    let mut sending = transfer();
    sending.sent = BTreeSet::from([1, 2]);
    sending.sender_completed = false;
    let invariants = ClusterInvariants::new(
        vec![sending],
        vec![
            node(1, vec![record(4, &[1, 2])]),
            node(2, vec![record(4, &[1, 2, 3])]),
        ],
    );

    // Act
    let violations = invariants.subsets();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn unique_names_name_two_transfers_under_one_name() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![transfer(), transfer()], cluster(&[1, 2]));

    // Act
    let violations = invariants.unique_names();

    // Assert
    assert_eq!(violations.len(), 1, "{violations:#?}");
}

#[test]
fn violations_and_abandonment_of_a_consistent_cluster_are_none() {
    // Arrange
    let invariants = ClusterInvariants::new(vec![transfer()], cluster(&[1, 2]));

    // Act
    let violations = invariants.violations();

    // Assert
    assert!(violations.is_empty(), "{violations:#?}");
    assert!(invariants.abandonment().is_empty());
}
