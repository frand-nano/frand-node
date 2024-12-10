use anyhow::Result;
use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use bases::{NodeId, NodeKey, Packet, Reporter, StateNode};
use crate::*;

mod frand_node {
    pub mod macro_prelude {
        pub use crate::macro_prelude::*;
    }
}

#[derive(Debug, Clone)]
pub struct TerminalNode<S: State> {
    key: NodeKey,
    reporter: Reporter,
    _phantom: PhantomData<S>,
}

impl<S: State + Default> Default for TerminalNode<S> {
    fn default() -> Self { Self::new(vec![], None, &(|_|()).into()) }
}

impl<S: State> Node for TerminalNode<S> {
    fn key(&self) -> &NodeKey { &self.key }
    fn reporter(&self) -> &Reporter { &self.reporter }

    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
        reporter: &Reporter,
    ) -> Self {
        if let Some(id) = id { key.push(id); }

        Self { 
            key: key.into_boxed_slice(),   
            reporter: reporter.clone(),
            _phantom: PhantomData::default(), 
        }
    }
}

impl<S: State> Emitter<S> for TerminalNode<S> {
    fn emit(&self, state: S) {
        self.reporter().report(state.into_packet(self.key().clone()))
    }
}

pub struct TerminalStateNode<'sn, S: State> {
    state: &'sn mut S,
    node: &'sn S::Node,
}

impl<S: State> TerminalStateNode<'_, S> {
    pub fn apply_state(&mut self, state: S) {
        *self.state = state;       
    }
}

impl<S: State> Deref for TerminalStateNode<'_, S> {
    type Target = S;
    fn deref(&self) -> &Self::Target { self.state }
}

impl<S: State> DerefMut for TerminalStateNode<'_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.state }
}

impl<S: State> Emitter<S> for TerminalStateNode<'_, S> 
where S::Node: Emitter<S> {
    fn emit(&self, state: S) { self.node.emit(state) }
}

impl<'sn, S: State> StateNode<'sn, S> for TerminalStateNode<'sn, S> 
where 
S::Node: Emitter<S>,
S: State<Message = S>,
{
    fn new(state: &'sn mut S, node: &'sn S::Node) -> Self { Self { state, node } }

    fn clone_state(&self) -> S { self.state.clone() }

    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => {
                let state: S = packet.read_state();    
                *self.state = state.clone();                
                Ok(state)
            },
        }        
    }
}

#[macro_export]
macro_rules! impl_node_for {
    ( $head: ty $(,$tys: ty)* $(,)? ) => { 
        impl_node_for!{ @inner($head, $($tys,)*) }
    };
    ( @inner($($tys: ty,)*) ) => {    
        $(
            impl frand_node::macro_prelude::Message for $tys {
                fn from_packet(
                    depth: usize,
                    packet: &frand_node::macro_prelude::Packet,
                ) -> frand_node::macro_prelude::anyhow::Result<Self> {
                    Ok(match packet.get_id(depth) {
                        Some(_) => Err(packet.error(depth, "unknown id")),
                        None => Ok(packet.read_state()),
                    }?)
                }
            }

            impl frand_node::macro_prelude::State for $tys {
                type Node = frand_node::macro_prelude::TerminalNode<Self>;
                type Message = Self;
                type StateNode<'sn> = frand_node::macro_prelude::TerminalStateNode<'sn, Self>;

                fn apply(
                    &mut self,  
                    depth: usize,
                    packet: &frand_node::macro_prelude::Packet,
                ) -> frand_node::macro_prelude::anyhow::Result<()> {
                    match packet.get_id(depth) {
                        Some(_) => Err(packet.error(depth, "unknown id")),
                        None => Ok(*self = packet.read_state()),
                    }
                }
            }
        )*      
    };
}

impl_node_for!{ 
    i8, i16, i32, i64, i128, 
    u8, u16, u32, u64, u128, 
    f32, f64,
    char, bool, (),
}