// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::rules::change::Change;
use crate::rules::command_situation::CommandSituation;
use crate::rules::registry_situation::RegistrySituation;
use crate::rules::rule::Rule;
use crate::rules::session_situation::SessionSituation;
use crate::rules::situation::Situation;

const RULES: [(&str, Rule); 25] = [
    (
        "no-chunks",
        Rule {
            situation: Situation::Command(CommandSituation::NoChunks),
            change: Change::Nothing,
        },
    ),
    (
        "self-addressed",
        Rule {
            situation: Situation::Command(CommandSituation::SelfAddressed),
            change: Change::Nothing,
        },
    ),
    (
        "not-a-party",
        Rule {
            situation: Situation::Command(CommandSituation::NotAParty),
            change: Change::Nothing,
        },
    ),
    (
        "start-registry-full",
        Rule {
            situation: Situation::Registry(RegistrySituation::Full),
            change: Change::Nothing,
        },
    ),
    (
        "start-started",
        Rule {
            situation: Situation::Always,
            change: Change::SessionStarted,
        },
    ),
    (
        "own-session",
        Rule {
            situation: Situation::Command(CommandSituation::OwnSession),
            change: Change::Nothing,
        },
    ),
    (
        "adopt-unaffected",
        Rule {
            situation: Situation::Session(SessionSituation::Alike),
            change: Change::Nothing,
        },
    ),
    (
        "adopt-conflicted",
        Rule {
            situation: Situation::Session(SessionSituation::Differing),
            change: Change::Nothing,
        },
    ),
    (
        "adopt-registry-full",
        Rule {
            situation: Situation::Registry(RegistrySituation::FullAndNotOpen),
            change: Change::Nothing,
        },
    ),
    (
        "adopt-started",
        Rule {
            situation: Situation::Always,
            change: Change::SessionAdopted,
        },
    ),
    (
        "empty-run",
        Rule {
            situation: Situation::Command(CommandSituation::EmptyRun),
            change: Change::Nothing,
        },
    ),
    (
        "partial-not-started",
        Rule {
            situation: Situation::Registry(RegistrySituation::NotOpen),
            change: Change::Nothing,
        },
    ),
    (
        "misattributed",
        Rule {
            situation: Situation::Session(SessionSituation::Misattributed),
            change: Change::Nothing,
        },
    ),
    (
        "wrong-side",
        Rule {
            situation: Situation::Session(SessionSituation::WrongSide),
            change: Change::Nothing,
        },
    ),
    (
        "chunk-out-of-range",
        Rule {
            situation: Situation::Session(SessionSituation::OutOfRange),
            change: Change::Nothing,
        },
    ),
    (
        "partial-advanced",
        Rule {
            situation: Situation::Session(SessionSituation::Advances),
            change: Change::RunAdded,
        },
    ),
    (
        "partial-completed",
        Rule {
            situation: Situation::Session(SessionSituation::Completes),
            change: Change::SessionCompleted,
        },
    ),
    (
        "partial-unaffected",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "cancel-cancelled",
        Rule {
            situation: Situation::Registry(RegistrySituation::Open),
            change: Change::SessionCancelled,
        },
    ),
    (
        "cancel-unaffected",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "query-not-started",
        Rule {
            situation: Situation::Registry(RegistrySituation::NeitherOpenNorRetained),
            change: Change::Nothing,
        },
    ),
    (
        "recovery-shown",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "missing-chunks-listed",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "recoveries-listed",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
    (
        "show-registry",
        Rule {
            situation: Situation::Always,
            change: Change::Nothing,
        },
    ),
];

// What the prose gives each row of the decision table: the situation in which
// it applies and the change it makes. The table gives the rest.
pub struct RuleBook;

impl RuleBook {
    #[must_use]
    pub fn row_ids() -> Vec<&'static str> {
        RULES.iter().map(|(id, _)| *id).collect()
    }

    #[must_use]
    pub fn rule(row_id: &str) -> Rule {
        RULES
            .iter()
            .find(|(id, _)| *id == row_id)
            .map(|(_, rule)| *rule)
            .expect("a row the model knows")
    }
}
