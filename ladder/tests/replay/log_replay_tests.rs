// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The replay tier: logs written in the specification's JSON, one step per call
// as the observer is told them, replayed into an instance created from the
// same origin -- every receipt rebuilt, a dropped line detected by the gap in
// the numbers, and the replay resynchronised from the next checkpoint the log
// holds.

use crate::support::log_replay::LogReplay;
use crate::support::random_commands::RandomCommands;
use crate::support::recording_observer::RecordingObserver;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_reader::SpecReader;
use crate::support::spec_schema::SpecSchema;
use crate::support::spec_writer::SpecWriter;
use beggar::beggar::Beggar;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::origin::Origin;
use beggar::session_id::SessionId;
use beggar_core_validation::scenario_reason::ScenarioReason;
use proptest::collection::vec;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use proptest::prelude::Just;
use proptest::prelude::ProptestConfig;
use proptest::prelude::Strategy;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

// The log an instance created fresh writes of the commands: one line per call,
// as its observer is told them.
fn log(fresh: &Fresh, commands: Vec<Command<ScenarioReason>>) -> Vec<Value> {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut beggar: Beggar<ScenarioReason> = Beggar::new(
        Origin::Fresh(fresh.clone()),
        Box::new(RecordingObserver::new(Rc::clone(&calls))),
    );
    for command in commands {
        beggar.process(command);
    }
    let lines = calls
        .borrow()
        .iter()
        .map(|(command, receipt)| SpecWriter::step(command, receipt))
        .collect();
    lines
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn replay_of_a_log_missing_a_line_detects_the_gap_and_resynchronises_at_the_next_checkpoint(
        (fresh, before, after, dropped) in RandomCommands::fresh()
            .prop_flat_map(|fresh| {
                let instance = fresh.instance;
                (
                    Just(fresh),
                    vec(RandomCommands::command(instance), 1..20),
                    vec(RandomCommands::command(instance), 0..20),
                )
            })
            .prop_flat_map(|(fresh, before, after)| {
                let dropped = 0..before.len();
                (Just(fresh), Just(before), Just(after), dropped)
            }),
    ) {
        // Arrange
        let commands: Vec<Command<ScenarioReason>> = before
            .into_iter()
            .chain([Command::ShowRegistry])
            .chain(after)
            .collect();
        let checkpoint = dropped
            + 1
            + commands[dropped + 1..]
                .iter()
                .position(|command| *command == Command::ShowRegistry)
                .expect("a checkpoint after the dropped line");
        let mut lines = log(&fresh, commands);
        let total = lines.len();
        lines.remove(dropped);

        // Act
        let report = LogReplay::new(Origin::Fresh(fresh)).replay(&lines);

        // Assert
        let number = dropped as u64 + 1;
        prop_assert_eq!(report.lines, total as u64 - 1);
        prop_assert_eq!(report.gaps, vec![number..=number]);
        prop_assert_eq!(
            report.reconstructed,
            (dropped + (total - 1 - checkpoint)) as u64
        );
        prop_assert!(report.departures.is_empty(), "{:?}", report.departures);
    }

    #[test]
    fn replay_of_any_log_rebuilds_every_receipt_from_lines_the_schema_admits(
        (fresh, commands) in RandomCommands::fresh_and_commands(60),
    ) {
        // Arrange
        let schema = SpecSchema::new(SpecFiles::schema());
        let lines = log(&fresh, commands);

        // Act
        let report = LogReplay::new(Origin::Fresh(fresh)).replay(&lines);

        // Assert
        prop_assert!(lines.iter().all(|line| schema.admits_step(line)));
        prop_assert_eq!(report.reconstructed, lines.len() as u64);
        prop_assert!(report.gaps.is_empty());
        prop_assert!(report.departures.is_empty(), "{:?}", report.departures);
    }
}

fn send(first: u64, last: u64) -> Command<ScenarioReason> {
    Command::SendRecoveryPartial {
        session_id: SessionId {
            node: 1,
            incarnation: 7,
            count: 1,
        },
        first,
        last,
        sender: 1,
        receiver: 2,
    }
}

#[test]
fn replay_of_a_log_missing_a_line_and_no_checkpoint_after_it_rebuilds_only_what_came_before() {
    // Arrange
    let fresh = Fresh {
        instance: InstanceId {
            node: 1,
            incarnation: 7,
        },
        retention: 0,
        bound: None,
    };
    let mut lines = log(
        &fresh,
        vec![
            Command::StartRecovery {
                chunks: 4,
                sender: 1,
                receiver: 2,
            },
            send(1, 1),
            send(2, 2),
            send(3, 3),
        ],
    );
    lines.remove(1);

    // Act
    let report = LogReplay::new(Origin::Fresh(fresh)).replay(&lines);

    // Assert
    assert_eq!(report.lines, 3);
    assert_eq!(report.reconstructed, 1);
    assert_eq!(report.gaps, vec![2..=2]);
    assert!(report.departures.is_empty());
}

#[test]
fn replay_of_every_vector_as_a_log_rebuilds_every_receipt() {
    // Arrange
    let vectors = SpecFiles::vectors();

    // Act
    let shortfalls: Vec<String> = vectors
        .iter()
        .filter_map(|(name, vector)| {
            let steps = vector["steps"].as_array().expect("a vector's steps");
            let report = LogReplay::new(SpecReader::origin(&vector["origin"])).replay(steps);
            let whole = report.reconstructed == steps.len() as u64
                && report.gaps.is_empty()
                && report.departures.is_empty();
            (!whole).then(|| {
                format!(
                    "{name}: rebuilt {} of {}, gaps {:?}, departures {:?}",
                    report.reconstructed, report.lines, report.gaps, report.departures
                )
            })
        })
        .collect();

    // Assert
    assert!(!vectors.is_empty(), "no vector was read");
    assert!(shortfalls.is_empty(), "{shortfalls:#?}");
}
