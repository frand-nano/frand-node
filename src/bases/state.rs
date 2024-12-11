use anyhow::Result;
use super::*;

pub trait State: 'static + Default + Clone + Emitable {
    type Anchor: Anchor;
    type Message: Message;
    type Node<'sn>: Node<'sn, Self>;

    fn new_anchor<R: Into<Reporter>>(
        reporter: R,
    ) -> Self::Anchor { 
        Self::Anchor::new(vec![], None, &reporter.into()) 
    }

    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()>;    

    fn with<'sn>(&'sn mut self, anchor: &'sn Self::Anchor) -> Self::Node<'sn> {
        Self::Node::new(self, anchor)
    }
}

impl<S: State> Emitable for S {
    fn into_packet(self, anchor_key: AnchorKey) -> Packet { 
        Packet::new(anchor_key, self) 
    }
}