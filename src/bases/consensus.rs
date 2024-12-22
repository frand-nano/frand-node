use super::*;

pub trait Consensus<S: State>: Sized + Clone + Default + Send + Sync {   
    fn new(
        key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self;

    fn new_node(&self, reporter: &Reporter) -> S::Node;
    fn clone_state(&self) -> S;
    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<(), PacketError>;   
    fn apply_state(&mut self, state: S);
    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message, PacketError>;    
}