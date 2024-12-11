use anyhow::Result;
use super::{Emitter, Packet, State};

pub trait Node<'n, S: State>: Emitter + Send + Sync {    
    fn new(state: &'n S, anchor: &'n S::Anchor) -> Self;
    fn clone_state(&self) -> S;
    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()>;    
    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message>;    
}