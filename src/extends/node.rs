use std::marker::PhantomData;
use bases::{NodeId, NodeKey, Reporter};
use crate::*;

mod frand_node {
    pub mod macro_prelude {
        pub use crate::macro_prelude::*;
    }
}

#[derive(Debug)]
pub struct TerminalNode<S: State> {
    key: NodeKey,
    reporter: Reporter,
    _phantom: PhantomData<S>,
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