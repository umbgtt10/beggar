// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::command::Command;
use crate::observer::Observer;
use crate::origin::Origin;
use crate::output::Output;
use crate::receipt::Receipt;
use crate::registry::Registry;
use crate::session_id::SessionId;
use crate::side::Side;
use alloc::boxed::Box;
use alloc::vec::Vec;

// The registry's one entry point: every command answered by a receipt, the
// first row of the specification's matrix that applies giving the output, and
// reported with it to the observer. The registry holds the state and what the
// commands do to it; this numbers the calls, sends each command where it
// belongs, and tells the observer. R is the caller's cancel-reason
// enumeration: recorded in the receipt of a cancel, never read.
pub struct Beggar<R> {
    registry: Registry,
    observer: Box<dyn Observer<R>>,
}

impl<R: Clone> Beggar<R> {
    pub fn new(origin: Origin, observer: Box<dyn Observer<R>>) -> Self {
        let state = match origin {
            Origin::Fresh(fresh) => Checkpoint {
                instance: fresh.instance,
                retention: fresh.retention,
                bound: fresh.bound,
                open: Vec::new(),
                retained: Vec::new(),
                name_count: 0,
                next_receipt: 1,
            },
            Origin::Checkpoint(checkpoint) => checkpoint,
        };
        Self {
            registry: Registry::new(state),
            observer,
        }
    }

    pub fn process(&mut self, command: Command<R>) -> Receipt<R> {
        let number = self.registry.count_call();
        let output = match &command {
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            } => self.start(*chunks, *sender, *receiver),
            Command::AdoptRecovery {
                session_id,
                chunks,
                sender,
                receiver,
            } => self.adopt(*session_id, *chunks, *sender, *receiver),
            Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            } => self.partial(
                Side::Sending,
                *session_id,
                *first,
                *last,
                *sender,
                *receiver,
            ),
            Command::ReceiveRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            } => self.partial(
                Side::Receiving,
                *session_id,
                *first,
                *last,
                *sender,
                *receiver,
            ),
            Command::CancelRecovery { session_id, reason } => self.cancel(*session_id, reason),
            Command::ListRecoveries => self.registry.query_rows().recoveries(),
            Command::ShowRecovery { session_id } => {
                self.registry.query_rows().recovery(*session_id)
            }
            Command::ListMissingChunks { session_id } => {
                self.registry.query_rows().missing_chunks(*session_id)
            }
            Command::ShowRegistry => Output::RegistryShown(self.registry.checkpoint()),
        };
        let receipt = Receipt { number, output };
        self.observer.observe(&command, &receipt);
        receipt
    }

    fn adopt(
        &mut self,
        session_id: SessionId,
        chunks: u64,
        sender: u64,
        receiver: u64,
    ) -> Output<R> {
        if let Some(output) = self
            .registry
            .opening_rows()
            .adoption(session_id, chunks, sender, receiver)
        {
            return output;
        }
        self.registry.open(session_id, chunks, sender, receiver);
        Output::RecoveryStarted { session_id }
    }

    // The cancel's reason recorded in its receipt when the session it names
    // was open, and evicted.
    fn cancel(&mut self, session_id: SessionId, reason: &R) -> Output<R> {
        if self.registry.evict(session_id) {
            Output::RecoveryCancelled(reason.clone())
        } else {
            Output::RecoveryUnaffected
        }
    }

    fn partial(
        &mut self,
        side: Side,
        session_id: SessionId,
        first: u64,
        last: u64,
        sender: u64,
        receiver: u64,
    ) -> Output<R> {
        let attribution = self
            .registry
            .partial_rows()
            .attribution(side, session_id, first, last, sender, receiver);
        match attribution {
            Ok(index) => self.registry.account(index, first, last),
            Err(output) => output,
        }
    }

    fn start(&mut self, chunks: u64, sender: u64, receiver: u64) -> Output<R> {
        if let Some(rejection) = self
            .registry
            .opening_rows()
            .start_refusal(chunks, sender, receiver)
        {
            return Output::RecoveryRejected(rejection);
        }
        let session_id = self.registry.mint();
        self.registry.open(session_id, chunks, sender, receiver);
        Output::RecoveryStarted { session_id }
    }
}
