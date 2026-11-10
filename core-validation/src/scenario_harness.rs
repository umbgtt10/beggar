// Copyright (c) 2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::scenario_node::ScenarioNode;
use crate::scenario_reason::ScenarioReason;
use alloc::vec::Vec;
use beggar::checkpoint::Checkpoint;
use beggar::command::Command;
use beggar::receipt::Receipt;
use beggar::session_id::SessionId;

// A deterministic scenario over several nodes, driven by hand: each call made
// to the node it belongs on, as a control layer would make it -- a start and a
// send on the sender, an adopt and a receive on the receiver -- or any command
// to any node, for what a control layer would not do.
pub struct ScenarioHarness {
    nodes: Vec<ScenarioNode>,
}

impl ScenarioHarness {
    pub fn new(nodes: &[u64]) -> Self {
        Self {
            nodes: nodes.iter().map(|node| ScenarioNode::new(*node)).collect(),
        }
    }

    pub fn adopt(
        &mut self,
        receiver: u64,
        session_id: SessionId,
        chunks: u64,
        sender: u64,
    ) -> Receipt<ScenarioReason> {
        self.process(
            receiver,
            Command::AdoptRecovery {
                session_id,
                chunks,
                sender,
                receiver,
            },
        )
    }

    pub fn checkpoint(&mut self, node: u64) -> Checkpoint {
        let index = self.index(node);
        self.nodes[index].checkpoint()
    }

    pub fn process(
        &mut self,
        node: u64,
        command: Command<ScenarioReason>,
    ) -> Receipt<ScenarioReason> {
        let index = self.index(node);
        self.nodes[index].process(command)
    }

    pub fn receive(
        &mut self,
        receiver: u64,
        session_id: SessionId,
        first: u64,
        last: u64,
        sender: u64,
    ) -> Receipt<ScenarioReason> {
        self.process(
            receiver,
            Command::ReceiveRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            },
        )
    }

    pub fn restart(&mut self, node: u64) {
        let index = self.index(node);
        self.nodes[index].restart();
    }

    pub fn send(
        &mut self,
        sender: u64,
        session_id: SessionId,
        first: u64,
        last: u64,
        receiver: u64,
    ) -> Receipt<ScenarioReason> {
        self.process(
            sender,
            Command::SendRecoveryPartial {
                session_id,
                first,
                last,
                sender,
                receiver,
            },
        )
    }

    pub fn start(&mut self, sender: u64, receiver: u64, chunks: u64) -> Receipt<ScenarioReason> {
        self.process(
            sender,
            Command::StartRecovery {
                chunks,
                sender,
                receiver,
            },
        )
    }

    fn index(&self, node: u64) -> usize {
        self.nodes
            .iter()
            .position(|scenario_node| scenario_node.instance().node == node)
            .unwrap_or_else(|| panic!("no node {node} in the scenario"))
    }
}
