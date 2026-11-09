// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::command::Command;
use crate::observer::Observer;
use crate::origin::Origin;
use crate::output::Output;
use crate::receipt::Receipt;
use crate::received_chunks::ReceivedChunks;
use crate::rejection::Rejection;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;

// The registry, and its one entry point: every command answered by a receipt,
// the first row of the specification's matrix that applies giving the output,
// and reported with it to the observer. Its state is kept in the checkpoint's
// own shape, so ShowRegistry answers it as it stands. R is the caller's
// cancel-reason enum, which no command carries yet.
pub struct Beggar<R> {
    state: Checkpoint,
    observer: Box<dyn Observer>,
    reason: PhantomData<fn() -> R>,
}

impl<R> Beggar<R> {
    pub fn new(origin: Origin, observer: Box<dyn Observer>) -> Self {
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
            state,
            observer,
            reason: PhantomData,
        }
    }

    pub fn process(&mut self, command: Command) -> Receipt {
        let number = self.state.next_receipt;
        self.state.next_receipt = number + 1;
        let output = match command {
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            } => self.start(chunks, sender, receiver),
            Command::ShowRegistry => Output::RegistryShown(self.state.clone()),
        };
        let receipt = Receipt { number, output };
        self.observer.observe(&command, &receipt);
        receipt
    }

    fn refusal(&self, chunks: u64, sender: u64, receiver: u64) -> Option<Rejection> {
        let node = self.state.instance.node;
        let open = self.state.open.len() as u64;
        if chunks == 0 {
            Some(Rejection::NoChunks)
        } else if sender == receiver {
            Some(Rejection::SelfAddressed)
        } else if node != sender && node != receiver {
            Some(Rejection::NotAParty)
        } else {
            self.state
                .bound
                .filter(|bound| open >= *bound)
                .map(|capacity| Rejection::RegistryFull { capacity })
        }
    }

    fn start(&mut self, chunks: u64, sender: u64, receiver: u64) -> Output {
        if let Some(rejection) = self.refusal(chunks, sender, receiver) {
            return Output::RecoveryRejected(rejection);
        }
        self.state.name_count += 1;
        let session_id = SessionId {
            node: self.state.instance.node,
            incarnation: self.state.instance.incarnation,
            count: self.state.name_count,
        };
        self.state.open.push(SessionRecord {
            session_id,
            chunks,
            sender,
            receiver,
            received: ReceivedChunks::new(),
        });
        Output::RecoveryStarted { session_id }
    }
}
