use anyhow::Result;
use serde::Serialize;
use super::*;

pub trait State: 'static + Default + Clone + Emitable {
    type Message: Message;
    type Node: Node<Self>;

    fn new_node<R: Into<Reporter>>(
        reporter: R,
    ) -> Self::Node { 
        Self::Node::new(vec![], None, &reporter.into()) 
    }

    fn new_node_from<R: Into<Reporter>>(
        node: &Self::Node,
        reporter: R,
    ) -> Self::Node { 
        Self::Node::new_from(node, &reporter.into()) 
    }

    fn apply(&mut self, depth: usize, packet: Packet) -> Result<()>;    
    fn apply_message(&mut self, message: Self::Message);    
}

impl<S: State + Serialize> Emitable for S {
    fn to_packet(&self, anchor_key: &NodeKey) -> Packet { 
        Packet::new(anchor_key, self.to_owned()) 
    }

    fn into_packet(self, anchor_key: &NodeKey) -> Packet { 
        Packet::new(anchor_key, self) 
    }
}