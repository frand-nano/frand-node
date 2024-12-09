use anyhow::Result;
use super::*;

pub trait State: 'static + Default + Emitable {
    type Node: NodeBase;
    type Message: Message;

    fn new_node<R: Into<Reporter>>(
        reporter: R,
    ) -> Self::Node { 
        Self::Node::new(vec![], None, &reporter.into()) 
    }

    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()>;
}

impl<S: State> Emitable for S {
    fn into_packet(self, node_key: NodeKey) -> Packet { 
        Packet::new(node_key, self) 
    }
}

impl<S: State> Emitter<S> for S::Node {
    fn emit(&self, state: S) -> Result<()> {
        self.reporter().report(state.into_packet(self.key().clone()))
    }
}