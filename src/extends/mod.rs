mod atomic_node;
mod terminal_node;
mod option_node;
mod vec_node;
mod container;
mod processor;

pub use self::{
    atomic_node::*,
    terminal_node::*,
    option_node::*,
    vec_node::*,
    container::*,
    processor::*,
};

mod frand_node {
    pub mod macro_prelude {
        pub use crate::macro_prelude::*;
    }
}

macro_rules! impl_state_for {
    ( $($tys: ty),+ $(,)? ) => {   
        $(
            impl frand_node::macro_prelude::State for $tys {
                type Message = Self;
                type Consensus<M: frand_node::macro_prelude::Message> = frand_node::macro_prelude::TerminalConsensus<M, Self>;
                type Node<M: frand_node::macro_prelude::Message> = frand_node::macro_prelude::TerminalNode<M, Self>;

                fn apply(
                    &mut self,  
                    message: Self::Message,
                ) {
                    *self = message;
                }
            }
        )*      
    };
}

macro_rules! impl_message_for {
    ( $($tys: ty),+ $(,)? ) => {   
        $(
            impl frand_node::macro_prelude::Message for $tys {
                fn from_state<S: frand_node::macro_prelude::State>(
                    header: &frand_node::macro_prelude::Header, 
                    depth: usize, 
                    state: S,
                ) -> core::result::Result<Self, frand_node::macro_prelude::MessageError> {                    
                    match header.get(depth).copied() {
                        Some(_) => Err(frand_node::macro_prelude::MessageError::new(
                            header.clone(),
                            depth,
                            "unknown id",
                        )),
                        None => Ok(
                            unsafe { Self::cast_state::<S, $tys>(state) }
                        ),
                    }
                }

                fn from_packet(
                    packet: &frand_node::macro_prelude::Packet, 
                    depth: usize, 
                ) -> core::result::Result<Self, frand_node::macro_prelude::PacketError> {
                    Ok(match packet.get_id(depth) {
                        Some(_) => Err(frand_node::macro_prelude::PacketError::new(
                            packet.clone(),
                            depth,
                            "unknown id",
                        )),
                        None => Ok(packet.read_state()),
                    }?)     
                }

                fn to_packet(
                    &self,
                    header: &frand_node::macro_prelude::Header, 
                ) -> core::result::Result<frand_node::macro_prelude::Packet, frand_node::macro_prelude::MessageError> {
                    Ok(frand_node::macro_prelude::Packet::new(
                        header.clone(), 
                        self,
                    ))
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