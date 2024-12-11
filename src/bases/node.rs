use anyhow::Result;
use super::{Emitter, Packet, State};

pub trait Node<'sn, S: State>: Emitter<S> {    
    fn new(state: &'sn S, anchor: &'sn S::Anchor) -> Self;
    fn clone_state(&self) -> S;
    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message>;    
}