// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::cluster_invariants::ClusterInvariants;
use crate::scenario_event::ScenarioEvent;
use crate::scenario_node::ScenarioNode;
use crate::scenario_reason::ScenarioReason;
use crate::scenario_trace_entry::ScenarioTraceEntry;
use crate::transfer::Transfer;
use crate::transport::fault_counts::FaultCounts;
use crate::transport::fault_policy::FaultPolicy;
use crate::transport::faulty_transport::FaultyTransport;
use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::output::Output;
use beggar::receipt::Receipt;
use beggar::session_id::SessionId;

// A cluster recovering from itself over a faulty transport, its nodes driven
// as their control layers would drive them. A pull is a request that opens
// nothing, answered with a session of as many chunks as the server holds, up
// to the number wanted; a push is a session started unprompted. A sender
// reports each run of a page of chunks as it sends it, one page a turn,
// turns alternating with deliveries; a receiver adopts the session from every
// run delivered to it, then receives the run. Its cancellation policy: a
// receiver cancels a session no run has reached for as many steps as its
// patience, if it has one, and every node cancels what it still holds when it
// is told the cluster is done -- a session stalled, with so many chunks held --
// telling the session's sender, which cancels the pages it still holds and
// sends no more of them.
// What the cluster knows of each transfer from outside its nodes is kept
// beside them, with every receipt checked against what the chunks sent and
// delivered, and the policy's cancels, call for.
pub struct ClusterSimulation {
    nodes: Vec<ScenarioNode>,
    transport: FaultyTransport,
    transfers: Vec<Transfer>,
    pending: VecDeque<ScenarioEvent>,
    page: u64,
    holding: u64,
    patience: Option<u64>,
    turn: u64,
}

impl ClusterSimulation {
    pub fn new(
        nodes: u64,
        policy: FaultPolicy,
        page: u64,
        holding: u64,
        patience: Option<u64>,
    ) -> Self {
        Self {
            nodes: (1..=nodes).map(ScenarioNode::new).collect(),
            transport: FaultyTransport::new(policy),
            transfers: Vec::new(),
            pending: VecDeque::new(),
            page,
            holding,
            patience,
            turn: 0,
        }
    }

    // The policy's last word: every session still open, at either end,
    // cancelled by the node holding it.
    pub fn cancel_abandoned(&mut self) {
        for index in 0..self.transfers.len() {
            if self.transfers[index].receiver_open {
                self.cancel(index);
            }
        }
    }

    // The invariants that hold at every step, checked now.
    pub fn check(&mut self) -> Vec<String> {
        ClusterInvariants::new(self.transfers.clone(), self.checkpoints()).violations()
    }

    // Abandonment, checked once nothing is in flight.
    pub fn check_quiescent(&mut self) -> Vec<String> {
        ClusterInvariants::new(self.transfers.clone(), self.checkpoints()).abandonment()
    }

    // Each node's checkpoint, in the order of the nodes.
    pub fn checkpoints(&mut self) -> Vec<Checkpoint> {
        self.nodes
            .iter_mut()
            .map(ScenarioNode::checkpoint)
            .collect()
    }

    pub fn counts(&self) -> FaultCounts {
        self.transport.counts()
    }

    pub fn heal(&mut self) {
        self.transport.heal();
    }

    pub fn pull(&mut self, requester: u64, server: u64, wanted: u64) {
        self.transport.send(ScenarioEvent::Request {
            requester,
            server,
            wanted,
        });
    }

    pub fn push(&mut self, sender: u64, receiver: u64, chunks: u64) {
        self.open(sender, receiver, chunks);
    }

    // The node restarted under its next incarnation: the pages it had still to
    // send are never sent, and the sessions it held are gone.
    pub fn restart(&mut self, node: u64) {
        let index = self.index(node);
        self.nodes[index].restart();
        self.pending.retain(|event| match event {
            ScenarioEvent::Run { sender, .. } => *sender != node,
            _ => true,
        });
        for transfer in &mut self.transfers {
            if transfer.sender == node && !transfer.sender_completed {
                transfer.interrupted = true;
            }
            if transfer.receiver == node && transfer.receiver_open {
                transfer.interrupted = true;
                transfer.receiver_open = false;
                transfer.held.clear();
            }
        }
    }

    // Steps until nothing is in flight, or the limit, checking the invariants
    // that hold at every step after each, and abandonment at the end.
    pub fn run_checked(&mut self, limit: u64) -> Vec<String> {
        let mut violations = Vec::new();
        let mut steps = 0;
        while self.step() {
            steps += 1;
            violations.extend(
                self.check()
                    .into_iter()
                    .map(|violation| format!("step {steps}: {violation}")),
            );
            if steps == limit {
                if self.busy() {
                    violations.push(format!("not settled after {limit} steps"));
                }
                return violations;
            }
        }
        violations.extend(self.check_quiescent());
        violations
    }

    // One page sent or one event delivered, turn about; false when there is
    // nothing to do.
    pub fn step(&mut self) -> bool {
        self.turn += 1;
        self.cancel_stalled();
        let sending_turn = self.turn % 2 == 1 || self.transport.in_flight() == 0;
        if let Some(run) = self.pending.front().copied().filter(|_| sending_turn) {
            self.pending.pop_front();
            self.send(run);
            return true;
        }
        if let Some(event) = self.transport.deliver() {
            self.deliver(event);
            return true;
        }
        if let Some(run) = self.pending.pop_front() {
            self.send(run);
            return true;
        }
        false
    }

    // Each node's trace, in the order of the nodes.
    pub fn traces(&self) -> Vec<Vec<ScenarioTraceEntry>> {
        self.nodes.iter().map(ScenarioNode::trace).collect()
    }

    pub fn transfers(&self) -> Vec<Transfer> {
        self.transfers.clone()
    }

    // A run delivered: the receiver adopts its session, then receives it.
    fn arrive(&mut self, run: ScenarioEvent) {
        let ScenarioEvent::Run {
            sender,
            receiver,
            session_id,
            chunks,
            first,
            last,
        } = run
        else {
            return;
        };
        let adopted = self.process(
            receiver,
            Command::AdoptRecovery {
                session_id,
                chunks,
                sender,
                receiver,
            },
        );
        let index = self.transfer(session_id);
        let transfer = &mut self.transfers[index];
        let finished = transfer.receiver_completions > 0 || transfer.cancellations > 0;
        if finished {
            transfer.late_copies += 1;
        }
        transfer.last_delivery = self.turn;
        match adopted.output {
            Output::RecoveryStarted { .. } if !transfer.receiver_open => {
                if finished {
                    transfer.reopens += 1;
                }
                transfer.receiver_open = true;
                transfer.held.clear();
            }
            Output::RecoveryUnaffected if transfer.receiver_open => {}
            _ => transfer.unsound += 1,
        }
        transfer.delivered.extend(first..=last);
        let received = self.process(
            receiver,
            Command::ReceiveRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            },
        );
        let transfer = &mut self.transfers[index];
        let before = transfer.held.len();
        transfer.held.extend(first..=last);
        let expected = Self::called_for(before, transfer.held.len(), chunks);
        if expected == Output::RecoveryCompleted {
            transfer.receiver_open = false;
            transfer.receiver_completions += 1;
        }
        if received.output != expected {
            transfer.unsound += 1;
        }
    }

    fn busy(&self) -> bool {
        !self.pending.is_empty() || self.transport.in_flight() > 0
    }

    // The output a run calls for, given how many chunks its end held before
    // and after it.
    fn called_for(before: usize, after: usize, chunks: u64) -> Output<ScenarioReason> {
        let held = after as u64;
        if after == before {
            Output::RecoveryUnaffected
        } else if held == chunks {
            Output::RecoveryCompleted
        } else {
            Output::RecoveryAdvanced { held, chunks }
        }
    }

    // A session cancelled by its receiver for stalling, with as many chunks
    // held as reached it since it opened there.
    fn cancel(&mut self, index: usize) {
        let transfer = &self.transfers[index];
        let (receiver, session_id) = (transfer.receiver, transfer.session_id);
        let reason = ScenarioReason::Stalled {
            held: transfer.held.len() as u64,
        };
        let cancelled = self.process(receiver, Command::CancelRecovery { session_id, reason });
        let transfer = &mut self.transfers[index];
        if cancelled.output != Output::RecoveryCancelled(reason) {
            transfer.unsound += 1;
        }
        transfer.receiver_open = false;
        transfer.held.clear();
        transfer.cancellations += 1;
        let sender = transfer.sender;
        self.transport.send(ScenarioEvent::Abandon {
            receiver,
            sender,
            session_id,
        });
    }

    // Every session a receiver holds that no run has reached for as many
    // steps as the patience, cancelled.
    fn cancel_stalled(&mut self) {
        let Some(patience) = self.patience else {
            return;
        };
        for index in 0..self.transfers.len() {
            let transfer = &self.transfers[index];
            if transfer.receiver_open && self.turn - transfer.last_delivery > patience {
                self.cancel(index);
            }
        }
    }

    fn deliver(&mut self, event: ScenarioEvent) {
        match event {
            ScenarioEvent::Request {
                requester,
                server,
                wanted,
            } => {
                let answer = wanted.min(self.holding);
                if answer > 0 {
                    self.open(server, requester, answer);
                }
            }
            ScenarioEvent::Run { .. } => self.arrive(event),
            ScenarioEvent::Abandon {
                sender, session_id, ..
            } => self.forsake(sender, session_id),
        }
    }

    // A sender told its session was given up: cancelled, with the pages it
    // still had to send, if it was still sending; unaffected if it had sent
    // them all, or restarted since.
    fn forsake(&mut self, sender: u64, session_id: SessionId) {
        let still_sending = self.pending.iter().any(|event| {
            matches!(event, ScenarioEvent::Run { session_id: pending, .. } if *pending == session_id)
        });
        let reason = ScenarioReason::PeerFailed;
        let cancelled = self.process(sender, Command::CancelRecovery { session_id, reason });
        let expected = if still_sending {
            Output::RecoveryCancelled(reason)
        } else {
            Output::RecoveryUnaffected
        };
        let index = self.transfer(session_id);
        let transfer = &mut self.transfers[index];
        if cancelled.output != expected {
            transfer.unsound += 1;
        }
        if still_sending {
            transfer.sender_cancelled = true;
            self.pending.retain(|event| {
                !matches!(event, ScenarioEvent::Run { session_id: pending, .. } if *pending == session_id)
            });
        }
    }

    fn index(&self, node: u64) -> usize {
        self.nodes
            .iter()
            .position(|scenario_node| scenario_node.instance().node == node)
            .unwrap_or_else(|| panic!("no node {node} in the cluster"))
    }

    // A session started on its sender, and its pages queued to send.
    fn open(&mut self, sender: u64, receiver: u64, chunks: u64) {
        let started = self.process(
            sender,
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            },
        );
        let Output::RecoveryStarted { session_id } = started.output else {
            return;
        };
        self.transfers.push(Transfer {
            session_id,
            sender,
            receiver,
            chunks,
            sent: BTreeSet::new(),
            delivered: BTreeSet::new(),
            held: BTreeSet::new(),
            receiver_open: false,
            sender_completed: false,
            receiver_completions: 0,
            reopens: 0,
            late_copies: 0,
            unsound: 0,
            interrupted: false,
            cancellations: 0,
            last_delivery: 0,
            sender_cancelled: false,
        });
        let mut first = 1;
        while first <= chunks {
            let last = (first + self.page - 1).min(chunks);
            self.pending.push_back(ScenarioEvent::Run {
                sender,
                receiver,
                session_id,
                chunks,
                first,
                last,
            });
            first = last + 1;
        }
    }

    fn process(&mut self, node: u64, command: Command<ScenarioReason>) -> Receipt<ScenarioReason> {
        let index = self.index(node);
        self.nodes[index].process(command)
    }

    // A page sent: reported on its sender, then handed to the transport.
    fn send(&mut self, run: ScenarioEvent) {
        let ScenarioEvent::Run {
            sender,
            receiver,
            session_id,
            chunks,
            first,
            last,
        } = run
        else {
            return;
        };
        let reported = self.process(
            sender,
            Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            },
        );
        let index = self.transfer(session_id);
        let transfer = &mut self.transfers[index];
        let before = transfer.sent.len();
        transfer.sent.extend(first..=last);
        let expected = Self::called_for(before, transfer.sent.len(), chunks);
        if expected == Output::RecoveryCompleted {
            transfer.sender_completed = true;
        }
        if reported.output != expected {
            transfer.unsound += 1;
        }
        self.transport.send(run);
    }

    fn transfer(&self, session_id: SessionId) -> usize {
        self.transfers
            .iter()
            .position(|transfer| transfer.session_id == session_id)
            .unwrap_or_else(|| panic!("no transfer {session_id:?}"))
    }
}
