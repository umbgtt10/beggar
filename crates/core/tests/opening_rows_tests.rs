// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::chunk_set::ChunkSet;
use beggar::instance_id::InstanceId;
use beggar::opening_rows::OpeningRows;
use beggar::output::Output;
use beggar::rejection::Rejection;
use beggar::session_id::SessionId;
use beggar::session_record::SessionRecord;

const INSTANCE: InstanceId = InstanceId {
    node: 1,
    incarnation: 7,
};
const PEER: u64 = 2;

// Sessions the peer started and this instance adopted, as many as asked.
fn adopted(sessions: u64) -> Vec<SessionRecord> {
    (1..=sessions)
        .map(|count| SessionRecord {
            session_id: peers(count),
            chunks: 4,
            sender: PEER,
            receiver: INSTANCE.node,
            chunk_set: ChunkSet::new(),
        })
        .collect()
}

fn peers(count: u64) -> SessionId {
    SessionId {
        node: PEER,
        incarnation: 3,
        count,
    }
}

#[test]
fn adoption_at_the_bound_is_registry_full_and_a_session_already_open_is_found_first() {
    // Arrange
    let open = adopted(2);
    let rows = OpeningRows::new(INSTANCE, &open, Some(2));

    // Act
    let new_session = rows.adoption::<()>(peers(3), 4, PEER, INSTANCE.node);
    let open_session = rows.adoption::<()>(peers(1), 4, PEER, INSTANCE.node);

    // Assert
    assert_eq!(
        new_session,
        Some(Output::RecoveryRejected(Rejection::RegistryFull {
            capacity: 2
        }))
    );
    assert_eq!(open_session, Some(Output::RecoveryUnaffected));
}

#[test]
fn adoption_between_two_other_nodes_is_not_a_party() {
    // Arrange
    let open = adopted(0);
    let rows = OpeningRows::new(INSTANCE, &open, None);

    // Act
    let adoption = rows.adoption::<()>(peers(1), 4, PEER, 3);

    // Assert
    assert_eq!(
        adoption,
        Some(Output::RecoveryRejected(Rejection::NotAParty))
    );
}

#[test]
fn adoption_of_a_name_from_this_nodes_previous_incarnation_is_none() {
    // Arrange
    let open = adopted(0);
    let rows = OpeningRows::new(INSTANCE, &open, None);
    let previous = SessionId {
        node: INSTANCE.node,
        incarnation: INSTANCE.incarnation - 1,
        count: 1,
    };

    // Act
    let adoption = rows.adoption::<()>(previous, 4, PEER, INSTANCE.node);

    // Assert
    assert_eq!(adoption, None);
}

#[test]
fn adoption_of_a_name_this_instance_minted_is_own_session_after_the_form_rows() {
    // Arrange
    let open = adopted(0);
    let rows = OpeningRows::new(INSTANCE, &open, None);
    let own = SessionId {
        node: INSTANCE.node,
        incarnation: INSTANCE.incarnation,
        count: 9,
    };

    // Act
    let well_formed = rows.adoption::<()>(own, 4, PEER, INSTANCE.node);
    let no_chunks = rows.adoption::<()>(own, 0, PEER, INSTANCE.node);

    // Assert
    assert_eq!(
        well_formed,
        Some(Output::RecoveryRejected(Rejection::OwnSession))
    );
    assert_eq!(
        no_chunks,
        Some(Output::RecoveryRejected(Rejection::NoChunks))
    );
}

#[test]
fn adoption_of_a_session_not_open_under_the_bound_is_none() {
    // Arrange
    let open = adopted(1);
    let rows = OpeningRows::new(INSTANCE, &open, Some(2));

    // Act
    let adoption = rows.adoption::<()>(peers(2), 4, PEER, INSTANCE.node);

    // Assert
    assert_eq!(adoption, None);
}

#[test]
fn adoption_of_an_open_session_on_other_terms_is_conflicted_carrying_the_open_sessions() {
    // Arrange
    let open = adopted(1);
    let rows = OpeningRows::new(INSTANCE, &open, None);

    // Act
    let other_count = rows.adoption::<()>(peers(1), 5, PEER, INSTANCE.node);
    let other_sender = rows.adoption::<()>(peers(1), 4, 3, INSTANCE.node);

    // Assert
    let open_terms = Some(Output::RecoveryConflicted {
        chunks: 4,
        sender: PEER,
        receiver: INSTANCE.node,
    });
    assert_eq!(other_count, open_terms);
    assert_eq!(other_sender, open_terms);
}

#[test]
fn start_refusal_at_the_bound_is_registry_full_carrying_it() {
    // Arrange
    let open = adopted(2);
    let rows = OpeningRows::new(INSTANCE, &open, Some(2));

    // Act
    let refusal = rows.start_refusal(4, INSTANCE.node, PEER);

    // Assert
    assert_eq!(refusal, Some(Rejection::RegistryFull { capacity: 2 }));
}

#[test]
fn start_refusal_between_two_other_nodes_is_not_a_party() {
    // Arrange
    let open = adopted(0);
    let rows = OpeningRows::new(INSTANCE, &open, None);

    // Act
    let refusal = rows.start_refusal(4, PEER, 3);

    // Assert
    assert_eq!(refusal, Some(Rejection::NotAParty));
}

#[test]
fn start_refusal_under_the_bound_is_none() {
    // Arrange
    let open = adopted(1);
    let rows = OpeningRows::new(INSTANCE, &open, Some(2));

    // Act
    let refusal = rows.start_refusal(4, PEER, INSTANCE.node);

    // Assert
    assert_eq!(refusal, None);
}

#[test]
fn start_refusal_with_no_chunks_is_no_chunks_before_any_other_row() {
    // Arrange
    let open = adopted(0);
    let rows = OpeningRows::new(INSTANCE, &open, Some(0));

    // Act
    let refusal = rows.start_refusal(0, PEER, PEER);

    // Assert
    assert_eq!(refusal, Some(Rejection::NoChunks));
}

#[test]
fn start_refusal_with_one_end_twice_is_self_addressed_before_not_a_party() {
    // Arrange
    let open = adopted(0);
    let rows = OpeningRows::new(INSTANCE, &open, None);

    // Act
    let refusal = rows.start_refusal(4, PEER, PEER);

    // Assert
    assert_eq!(refusal, Some(Rejection::SelfAddressed));
}
