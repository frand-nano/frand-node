use serde::{de::DeserializeOwned, Serialize};
use super::*;

pub trait State: 'static + Default + Clone + Sized + Serialize + DeserializeOwned + Emitable {
    type Message: Message;
    type Consensus: Consensus<Self>;
    type Node: Node<Self>;

    fn apply(&mut self, depth: usize, packet: Packet) -> Result<(), PacketError>;    
    fn apply_message(&mut self, message: Self::Message);    
}

impl<S: State> Emitable for S {
    fn to_packet(&self, node_key: &NodeKey) -> Packet { 
        Packet::new(node_key, self.to_owned()) 
    }

    fn into_packet(self, node_key: &NodeKey) -> Packet { 
        Packet::new(node_key, self) 
    }
}