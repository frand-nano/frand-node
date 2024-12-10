use anyhow::Result;
use super::{Emitter, Packet, State};

pub trait StateNode<'sn, S: State>: Emitter<S> {    
    fn new(state: &'sn mut S, node: &'sn S::Node) -> Self;
    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()>;    
}