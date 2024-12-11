use anyhow::Result;
use serde::Serialize;
use super::*;

pub trait State: 'static + Default + Clone + Emitable {
    type Anchor: Anchor;
    type Message: Message;
    type Node<'n>: Node<'n, Self>;

    fn new_anchor<R: Into<Reporter>>(
        reporter: R,
    ) -> Self::Anchor { 
        Self::Anchor::new(vec![], None, &reporter.into()) 
    }

    fn apply(&mut self, depth: usize, packet: Packet) -> Result<()>;    
    fn apply_message(&mut self, message: Self::Message);    

    fn with<'n>(&'n self, anchor: &'n Self::Anchor) -> Self::Node<'n> {
        Self::Node::new(self, anchor)
    }
}

impl<S: State + Serialize> Emitable for S {
    fn to_packet(&self, anchor_key: &AnchorKey) -> Packet { 
        Packet::new(anchor_key, self.to_owned()) 
    }

    fn into_packet(self, anchor_key: &AnchorKey) -> Packet { 
        Packet::new(anchor_key, self) 
    }
}