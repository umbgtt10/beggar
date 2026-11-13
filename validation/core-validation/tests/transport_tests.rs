// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use beggar::session_id::SessionId;
use beggar_core_validation::scenario_event::ScenarioEvent;
use beggar_core_validation::transport::fault_counts::FaultCounts;
use beggar_core_validation::transport::fault_policy::FaultPolicy;
use beggar_core_validation::transport::faulty_transport::FaultyTransport;
use beggar_core_validation::transport::message_kind::MessageKind;
use std::collections::BTreeSet;

fn delivered(transport: &mut FaultyTransport) -> Vec<ScenarioEvent> {
    let mut events = Vec::new();
    while let Some(event) = transport.deliver() {
        events.push(event);
    }
    events
}

fn request(requester: u64, server: u64) -> ScenarioEvent {
    ScenarioEvent::Request {
        requester,
        server,
        wanted: 1,
    }
}

fn run(sender: u64, receiver: u64) -> ScenarioEvent {
    ScenarioEvent::Run {
        sender,
        receiver,
        session_id: SessionId {
            node: sender,
            incarnation: 1,
            count: 1,
        },
        chunks: 1,
        first: 1,
        last: 1,
    }
}

#[test]
fn deliver_without_faults_hands_events_over_in_the_order_sent() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy::default());
    let sent: Vec<ScenarioEvent> = (1..=5).map(|node| request(node, 9)).collect();
    for event in &sent {
        transport.send(*event);
    }

    // Act
    let events = delivered(&mut transport);

    // Assert
    assert_eq!(events, sent);
    assert_eq!(transport.counts(), FaultCounts::default());
}

#[test]
fn of_each_event_names_its_kind() {
    // Arrange
    let abandon = ScenarioEvent::Abandon {
        receiver: 1,
        sender: 2,
        session_id: SessionId {
            node: 2,
            incarnation: 1,
            count: 1,
        },
    };
    let events = [request(1, 2), run(2, 1), abandon];

    // Act
    let kinds = events.map(|event| MessageKind::of(&event));

    // Assert
    assert_eq!(
        kinds,
        [MessageKind::Request, MessageKind::Run, MessageKind::Abandon]
    );
}

#[test]
fn send_across_a_full_partition_is_cut_until_the_transport_heals() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        partition: 100,
        ..FaultPolicy::default()
    });
    transport.send(request(1, 2));

    // Act
    transport.heal();
    transport.send(request(1, 2));

    // Assert
    assert_eq!(delivered(&mut transport), vec![request(1, 2)]);
    assert_eq!(transport.counts().partitioned, 1);
}

#[test]
fn send_at_a_tenth_loss_drops_about_a_tenth() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        loss: 10,
        seed: 42,
        ..FaultPolicy::default()
    });

    // Act
    for _ in 0..1000 {
        transport.send(request(1, 2));
    }

    // Assert
    let lost = transport.counts().lost;
    assert!((60..=140).contains(&lost), "{lost} of 1000 lost");
    assert_eq!(delivered(&mut transport).len() as u64, 1000 - lost);
}

#[test]
fn send_over_asymmetric_cuts_drops_some_links_in_one_direction_only() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        asymmetric: 50,
        seed: 7,
        ..FaultPolicy::default()
    });
    let nodes = 1..=5u64;
    for requester in nodes.clone() {
        for server in nodes.clone().filter(|server| *server != requester) {
            transport.send(request(requester, server));
        }
    }

    // Act
    let links: BTreeSet<(u64, u64)> = delivered(&mut transport)
        .into_iter()
        .map(|event| match event {
            ScenarioEvent::Request {
                requester, server, ..
            } => (requester, server),
            other => panic!("only requests were sent, not {other:?}"),
        })
        .collect();

    // Assert
    let one_way = links
        .iter()
        .filter(|(from, to)| !links.contains(&(*to, *from)))
        .count();
    assert!(one_way > 0, "no link cut in one direction only: {links:?}");
    assert!(transport.counts().asymmetric > 0);
}

#[test]
fn send_with_full_delay_still_delivers_every_event_and_counts_it() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        delay: 100,
        ..FaultPolicy::default()
    });

    // Act
    for node in 1..=4 {
        transport.send(request(node, 9));
    }

    // Assert
    assert_eq!(transport.in_flight(), 4);
    assert_eq!(delivered(&mut transport).len(), 4);
    assert_eq!(transport.counts().delayed, 4);
}

#[test]
fn send_with_full_duplication_delivers_every_event_twice_and_counts_it() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        duplication: 100,
        ..FaultPolicy::default()
    });

    // Act
    transport.send(request(1, 2));
    transport.send(request(3, 2));

    // Assert
    assert_eq!(
        delivered(&mut transport),
        vec![request(1, 2), request(1, 2), request(3, 2), request(3, 2)]
    );
    assert_eq!(transport.counts().duplicated, 2);
}

#[test]
fn send_with_full_loss_drops_every_event_and_counts_it() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        loss: 100,
        ..FaultPolicy::default()
    });

    // Act
    for node in 1..=3 {
        transport.send(request(node, 9));
    }

    // Assert
    assert!(delivered(&mut transport).is_empty());
    assert_eq!(transport.counts().lost, 3);
}

#[test]
fn send_with_reordering_delivers_every_event_out_of_order_and_counts_it() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        reorder: 50,
        seed: 3,
        ..FaultPolicy::default()
    });
    let sent: Vec<ScenarioEvent> = (1..=20).map(|node| request(node, 99)).collect();
    for event in &sent {
        transport.send(*event);
    }

    // Act
    let events = delivered(&mut transport);

    // Assert
    assert_ne!(events, sent);
    let mut sorted = events.clone();
    sorted.sort_by_key(|event| match event {
        ScenarioEvent::Request { requester, .. } => *requester,
        _ => 0,
    });
    assert_eq!(sorted, sent);
    assert!(transport.counts().reordered > 0);
}

#[test]
fn send_with_selective_loss_of_runs_drops_runs_and_lets_requests_through() {
    // Arrange
    let mut transport = FaultyTransport::new(FaultPolicy {
        selective: 100,
        selective_target: MessageKind::Run,
        ..FaultPolicy::default()
    });

    // Act
    transport.send(run(2, 1));
    transport.send(request(1, 2));

    // Assert
    assert_eq!(delivered(&mut transport), vec![request(1, 2)]);
    assert_eq!(transport.counts().selective, 1);
}
