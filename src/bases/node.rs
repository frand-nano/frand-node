use anyhow::Result;
use super::{NodeId, NodeKey, Emitter, Packet, Reporter, State};

pub trait Node<S: State>: 'static + Clone + Send + Sync + Emitter {    
    fn key(&self) -> &NodeKey;
    fn reporter(&self) -> &Reporter;
    
    fn new(
        key: Vec<NodeId>,
        id: Option<NodeId>,
        reporter: &Reporter,
    ) -> Self;
    
    fn new_from(
        node: &Self,
        reporter: &Reporter,
    ) -> Self;

    fn clone_state(&self) -> S;
    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()>;    
    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message>;    
}