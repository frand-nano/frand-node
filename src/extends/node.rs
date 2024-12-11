use anyhow::Result;
use serde::de::DeserializeOwned;
use std::{future::Future, sync::{Arc, RwLock, RwLockReadGuard}};
use bases::{NodeId, NodeKey, Packet, Reporter, Node};
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
    state: Arc<RwLock<S>>,
}

impl<S: State + Default> Default for TerminalNode<S> 
where S: State<Message = S> + DeserializeOwned {    
    fn default() -> Self { Self::new(vec![], None, &Reporter::None) }
}

impl<S: State> TerminalNode<S> {
    pub fn v<'a>(&'a self) -> RwLockReadGuard<'a, S> { 
        self.state.read().unwrap() 
    }

    pub fn apply_state(&mut self, state: S) {
        *self.state.write().unwrap() = state;       
    }
}

impl<S: State> Emitter for TerminalNode<S> {
    fn emit<E: 'static + Emitable>(&self, emitable: E) {
        self.reporter.report(&self.key, emitable)
    }
    
    fn emit_future<Fu, E>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = E> + Send,
    E: 'static + Emitable + Sized,
    {
        self.reporter.report_future(&self.key, future)
    }
}

impl<S> Node<S> for TerminalNode<S> 
where S: State<Message = S> + DeserializeOwned {    
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
            state: Arc::new(RwLock::new(S::default())),
        }
    }

    fn new_from(
        node: &Self,
        reporter: &Reporter,
    ) -> Self {
        Self { 
            key: node.key.clone(),   
            reporter: reporter.clone(),
            state: node.state.clone(),
        }
    }

    fn clone_state(&self) -> S { self.state.read().unwrap().clone() }

    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<()> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => Ok(*self.state.write().unwrap() = packet.read_state()),
        }        
    }

    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => {
                let state: S = packet.read_state();    
                *self.state.write().unwrap() = state.clone();                
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
                type Message = Self;
                type Node = frand_node::macro_prelude::TerminalNode<Self>;

                fn apply(
                    &mut self, 
                    depth: usize, 
                    packet: frand_node::macro_prelude::Packet,
                ) -> frand_node::macro_prelude::anyhow::Result<()>  {
                    match packet.get_id(depth) {
                        Some(_) => Err(packet.error(depth, "unknown id")),
                        None => Ok(*self = packet.read_state()),
                    }
                }    

                fn apply_message(
                    &mut self,  
                    message: Self::Message,
                ) {
                    *self = message;
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