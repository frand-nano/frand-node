mod atomic_node;
mod terminal_node;
mod option_node;
mod vec_node;
mod processor;
mod proxy;

pub use self::{
    atomic_node::*,
    terminal_node::*,
    option_node::*,
    vec_node::*,
    processor::*,
    proxy::*,
};

mod frand_node {
    pub mod macro_prelude {
        pub use crate::macro_prelude::*;
    }
}

macro_rules! impl_state_for {
    ( $($tys: ty),+ $(,)? ) => {   
        $(
            impl frand_node::macro_prelude::Accessor for $tys {
                type State = Self;
                type Message = Self;
                type Node = frand_node::macro_prelude::TerminalNode<Self>;
            }

            impl frand_node::macro_prelude::Emitable for $tys {}

            impl frand_node::macro_prelude::State for $tys {
                const NODE_SIZE: frand_node::macro_prelude::IdDelta = 1; 

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
                fn from_packet_message(
                    parent_key: frand_node::macro_prelude::Key,
                    depth: frand_node::macro_prelude::Depth,
                    packet: &frand_node::macro_prelude::PacketMessage, 
                ) -> core::result::Result<Self, frand_node::macro_prelude::MessageError> { 
                    match packet.key().id() - parent_key.id() {
                        0 => Ok(unsafe { 
                            frand_node::macro_prelude::State::from_emitable(packet.payload()) 
                        }),
                        id_delta => Err(frand_node::macro_prelude::MessageError::new(
                            packet.key(),
                            Some(id_delta),
                            format!(
                                "{}: unknown id_delta, depth: {:?}", 
                                std::any::type_name::<Self>(), 
                                depth,
                            ),
                        )),
                    }
                }

                fn from_packet(
                    parent_key: frand_node::macro_prelude::Key,
                    depth: frand_node::macro_prelude::Depth,
                    packet: &frand_node::macro_prelude::Packet, 
                ) -> core::result::Result<Self, frand_node::macro_prelude::PacketError> {
                    Ok(match packet.key().id() - parent_key.id() {
                        0 => Ok(packet.read_state()),
                        id_delta => Err(frand_node::macro_prelude::PacketError::new(
                            packet.clone(),
                            Some(id_delta),
                            format!(
                                "{}: unknown id_delta, depth: {:?}", 
                                std::any::type_name::<Self>(), 
                                depth,
                            ),
                        )),
                    }?)     
                }

                fn to_packet(
                    &self,
                    key: frand_node::macro_prelude::Key, 
                ) -> core::result::Result<frand_node::macro_prelude::Packet, frand_node::macro_prelude::MessageError> {
                    Ok(frand_node::macro_prelude::Packet::new(
                        key, 
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