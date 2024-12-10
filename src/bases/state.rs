use anyhow::Result;
use super::*;

pub trait State: 'static + Default + Emitable {
    type Node: Node;
    type Message: Message;
    type StateNode<'sn>: StateNode<'sn, Self>;

    fn new_node<R: Into<Reporter>>(
        reporter: R,
    ) -> Self::Node { 
        Self::Node::new(vec![], None, &reporter.into()) 
    }

    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()>;    

    fn with<'sn>(&'sn mut self, node: &'sn Self::Node) -> Self::StateNode<'sn> {
        Self::StateNode::new(self, node)
    }
}

impl<S: State> Emitable for S {
    fn into_packet(self, node_key: NodeKey) -> Packet { 
        Packet::new(node_key, self) 
    }
}