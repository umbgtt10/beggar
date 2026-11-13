// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::replay_report::ReplayReport;
use crate::support::spec_reader::SpecReader;
use crate::support::spec_writer::SpecWriter;
use beggar::beggar::Beggar;
use beggar::no_op_observer::NoOpObserver;
use beggar::origin::Origin;
use beggar_core_validation::scenario_reason::ScenarioReason;
use serde_json::Value;

// A log -- the specification's steps, one line per call -- replayed into an
// instance created from the origin the log was written from: each line's
// command processed and the receipt compared with the line's. A gap in the
// numbers proves lines lost; after one, nothing can be verified until a logged
// checkpoint, from which the replay resynchronises.
pub struct LogReplay {
    origin: Origin,
}

impl LogReplay {
    pub fn new(origin: Origin) -> Self {
        Self { origin }
    }

    pub fn replay(&self, lines: &[Value]) -> ReplayReport {
        let mut report = ReplayReport {
            lines: lines.len() as u64,
            reconstructed: 0,
            gaps: Vec::new(),
            departures: Vec::new(),
        };
        let mut expected = match &self.origin {
            Origin::Fresh(_) => 1,
            Origin::Checkpoint(checkpoint) => checkpoint.next_receipt,
        };
        let mut instance = Some(Self::instance(self.origin.clone()));
        for line in lines {
            let number = Self::number(&line["receipt"]["number"]);
            if number != expected {
                report.gaps.push(expected..=number - 1);
                instance = None;
            }
            expected = number + 1;
            instance = match instance {
                Some(mut beggar) => {
                    let receipt =
                        SpecWriter::receipt(&beggar.process(SpecReader::command(&line["command"])));
                    if receipt == line["receipt"] {
                        report.reconstructed += 1;
                    } else {
                        report.departures.push(format!(
                            "line {number}: replayed {receipt}, logged {}",
                            line["receipt"]
                        ));
                    }
                    Some(beggar)
                }
                None => Self::resynchronised(&line["receipt"]["output"]),
            };
        }
        report
    }

    fn instance(origin: Origin) -> Beggar<ScenarioReason> {
        Beggar::new(origin, Box::new(NoOpObserver))
    }

    fn number(value: &Value) -> u64 {
        value
            .as_str()
            .and_then(|digits| digits.parse().ok())
            .unwrap_or_else(|| panic!("a receipt's number is a string of decimal digits: {value}"))
    }

    // An instance restored from the checkpoint a line logged, if it logged one.
    fn resynchronised(output: &Value) -> Option<Beggar<ScenarioReason>> {
        output.get("RegistryShown").map(|checkpoint| {
            Self::instance(Origin::Checkpoint(SpecReader::checkpoint(checkpoint)))
        })
    }
}
