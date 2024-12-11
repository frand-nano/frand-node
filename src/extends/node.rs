use anyhow::Result;
use std::{borrow::Cow, marker::PhantomData, ops::Deref};
use bases::{AnchorId, AnchorKey, Packet, Reporter, Node};
use crate::*;

mod frand_node {
    pub mod macro_prelude {
        pub use crate::macro_prelude::*;
    }
}

#[derive(Debug, Clone)]
pub struct TerminalAnchor<S: State> {
    key: AnchorKey,
    reporter: Reporter,
    _phantom: PhantomData<S>,
}

impl<S: State + Default> Default for TerminalAnchor<S> {
    fn default() -> Self { Self::new(vec![], None, &(|_|()).into()) }
}

impl<S: State> Anchor for TerminalAnchor<S> {
    fn key(&self) -> &AnchorKey { &self.key }
    fn reporter(&self) -> &Reporter { &self.reporter }

    fn new(
        mut key: Vec<AnchorId>,
        id: Option<AnchorId>,
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

impl<S: State> Emitter<S> for TerminalAnchor<S> {
    fn emit(&self, state: S) {
        self.reporter().report(state.into_packet(self.key().clone()))
    }
}

pub struct TerminalNode<'sn, S: State> {
    state: Cow<'sn, S>,
    anchor: &'sn S::Anchor,
}

impl<S: State> TerminalNode<'_, S> {
    pub fn apply_state(&mut self, state: S) {
        *self.state.to_mut() = state;       
    }
}

impl<S: State> Deref for TerminalNode<'_, S> {
    type Target = S;
    fn deref(&self) -> &Self::Target { &self.state }
}

impl<S: State> Emitter<S> for TerminalNode<'_, S> 
where S::Anchor: Emitter<S> {
    fn emit(&self, state: S) { self.anchor.emit(state) }
}

impl<'sn, S: State> Node<'sn, S> for TerminalNode<'sn, S> 
where 
S::Anchor: Emitter<S>,
S: State<Message = S>,
{
    fn new(state: &'sn S, anchor: &'sn S::Anchor) -> Self { 
        Self { state: Cow::Borrowed(state), anchor } 
    }

    fn clone_state(&self) -> S { self.state.clone().into_owned() }

    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => {
                let state: S = packet.read_state();    
                *self.state.to_mut() = state.clone();                
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
                type Anchor = frand_node::macro_prelude::TerminalAnchor<Self>;
                type Message = Self;
                type Node<'sn> = frand_node::macro_prelude::TerminalNode<'sn, Self>;

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