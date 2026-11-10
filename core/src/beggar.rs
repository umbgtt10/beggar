// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::checkpoint::Checkpoint;
use crate::chunk_set::ChunkSet;
use crate::command::Command;
use crate::observer::Observer;
use crate::origin::Origin;
use crate::output::Output;
use crate::partial_rows::PartialRows;
use crate::receipt::Receipt;
use crate::session_id::SessionId;
use crate::session_record::SessionRecord;
use crate::start_rows::StartRows;
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
            Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            } => self.send(session_id, first, last, sender, receiver),
            Command::ShowRegistry => Output::RegistryShown(self.state.clone()),
        };
        let receipt = Receipt { number, output };
        self.observer.observe(&command, &receipt);
        receipt
    }

    // A partial's chunks, once it is attributed: none new leaves the session
    // unaffected, a full set completes and evicts it, and anything else
    // advances it.
    fn account(&mut self, index: usize, first: u64, last: u64) -> Output {
        let record = &mut self.state.open[index];
        let before = record.chunk_set.count();
        record.chunk_set.add(first, last);
        let held = record.chunk_set.count();
        let chunks = record.chunks;
        if held == before {
            Output::RecoveryUnaffected
        } else if held == chunks {
            self.state.open.remove(index);
            Output::RecoveryCompleted
        } else {
            Output::RecoveryAdvanced { held, chunks }
        }
    }

    fn send(
        &mut self,
        session_id: SessionId,
        first: u64,
        last: u64,
        sender: u64,
        receiver: u64,
    ) -> Output {
        let rows = PartialRows::new(self.state.instance.node, &self.state.open);
        match rows.attribution(session_id, first, last, sender, receiver) {
            Ok(index) => self.account(index, first, last),
            Err(output) => output,
        }
    }

    fn start(&mut self, chunks: u64, sender: u64, receiver: u64) -> Output {
        let rows = StartRows::new(
            self.state.instance.node,
            self.state.open.len() as u64,
            self.state.bound,
        );
        if let Some(rejection) = rows.refusal(chunks, sender, receiver) {
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
            chunk_set: ChunkSet::new(),
        });
        Output::RecoveryStarted { session_id }
    }
}
