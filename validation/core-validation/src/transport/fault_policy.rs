// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::transport::message_kind::MessageKind;

// One 0..=100 percentage per transport fault, as faction's policy has, and
// the seed that makes every misbehaving decision reproducible. Loss,
// duplication, delay, reordering and the selective fault, aimed at one kind
// of event, are rolled per event; partition and asymmetric are cuts per link,
// their percentage the share of links cut, lasting until the transport heals.
// Its default injects nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FaultPolicy {
    pub loss: u8,
    pub duplication: u8,
    pub delay: u8,
    pub reorder: u8,
    pub partition: u8,
    pub asymmetric: u8,
    pub selective: u8,
    pub selective_target: MessageKind,
    pub seed: u64,
}
