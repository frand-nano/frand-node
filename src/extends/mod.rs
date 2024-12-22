mod atomic_node;
mod terminal_node;
mod option_node;
mod vec_node;
mod container;
mod processor;
mod async_processor;

pub use self::{
    atomic_node::*,
    terminal_node::*,
    option_node::*,
    vec_node::*,
    container::*,
    processor::*,
    async_processor::*,
};

mod frand_node {
    pub mod macro_prelude {
        pub use crate::macro_prelude::*;
    }
}

#[macro_export]
macro_rules! impl_state_for {
    ( $($tys: ty,)+ ) => {   
        $(
            impl frand_node::macro_prelude::State for $tys {
                type Message = Self;
                type Consensus = frand_node::macro_prelude::TerminalConsensus<Self>;
                type Node = frand_node::macro_prelude::TerminalNode<Self>;

                fn apply(
                    &mut self, 
                    depth: usize, 
                    packet: frand_node::macro_prelude::Packet,
                ) -> core::result::Result<(), frand_node::macro_prelude::PacketError>  {
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

#[macro_export]
macro_rules! impl_message_for {
    ( $($tys: ty,)+ ) => {   
        $(
            impl frand_node::macro_prelude::Message for $tys {
                fn from_packet(
                    depth: usize,
                    packet: &frand_node::macro_prelude::Packet,
                ) -> core::result::Result<Self, frand_node::macro_prelude::PacketError> {
                    match packet.get_id(depth) {
                        Some(_) => Err(packet.error(depth, "unknown id")),
                        None => Ok(packet.read_state()),
                    }
                }
            }
        )*      
    };
}

impl_state_for!{ 
    i128, 
    u128, 
    f32, f64,
    char, (),
}

impl_message_for!{ 
    i8, i16, i32, i64, i128, 
    u8, u16, u32, u64, u128, 
    f32, f64,
    char, bool, (),
}