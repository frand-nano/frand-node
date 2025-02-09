use std::ops::Deref;
use std::sync::{Arc, RwLockReadGuard};
use crate::ext::*;
use crate::frand_node;

pub mod terminal {
    pub use super::*;

    #[derive(Debug, Clone)]
    pub struct Emitter<S: System> {
        callback: Callback<S>,
    }

    #[derive(Debug, Clone)]
    pub struct Accesser<S: System, CS: System> {
        access: RcAccess<S, CS>,
    }
    
    #[derive(Debug)]
    pub struct Node<'n, S: System, CS: System> {
        emitter: &'n S::Emitter,
        accesser: &'n S::Accesser<CS>,
        consensus: &'n Arc<RwLockReadGuard<'n, CS>>,
        transient: &'n Transient,      
    }
    
    impl<S: System> super::Emitter<S> for Emitter<S> 
    where S: System<Message = S> {    
        fn callback(&self) -> &Callback<S> { &self.callback }

        fn new(
            callback: Callback<S>,
        ) -> Self {
            Self { 
                callback, 
            }
        }
    }

    impl<S: System, CS: System> Deref for Accesser<S, CS> {
        type Target = RcAccess<S, CS>;
        fn deref(&self) -> &Self::Target { &self.access }
    }
        
    impl<S: System, CS: System> super::Accesser<S, CS> for Accesser<S, CS> {
        fn new(
            access: RcAccess<S, CS>,
        ) -> Self {
            Self { 
                access, 
            }
        }
    }

    impl<'n, S: System, CS: System> Deref for Node<'n, S, CS> 
    where S: System<Accesser<CS> = Accesser<S, CS>> {
        type Target = S;
        fn deref(&self) -> &Self::Target { 
            (self.accesser.access)(self.consensus, *self.transient)
        }
    }
    
    impl<'n, S: System, CS: System> super::Node<'n, S> for Node<'n, S, CS> 
    where S: System<Accesser<CS> = Accesser<S, CS>> {
        fn transient(&self) -> &Transient { self.transient }
        fn emitter(&self) -> &S::Emitter { &self.emitter }
    }
    
    impl<'n, S: System, CS: System> NewNode<'n, S, CS> for Node<'n, S, CS> {
        fn new(
            emitter: &'n S::Emitter,
            accesser: &'n S::Accesser<CS>,
            consensus: &'n Arc<RwLockReadGuard<'n, CS>>,
            transient: &'n Transient,        
        ) -> Self {
            Self { 
                emitter,
                accesser,
                consensus,
                transient,
            }
        }
        
        fn alt(
            &self,
            transient: Transient,           
        ) -> ConsensusRead<'n, S, CS> {
            ConsensusRead::new(
                self.emitter, 
                self.accesser, 
                self.consensus.clone(), 
                transient,
            )
        }
    }
}

#[macro_export]
macro_rules! impl_terminal_state_for {
    ( $($tys: ty),+ $(,)? ) => {   
        $(
            impl frand_node::ext::State for $tys {
                const NODE_SIZE: IdSize = 1;
                const NODE_ALT_SIZE: AltSize = 0;

                type Message = Self;
                type Emitter = frand_node::ext::terminal::Emitter<Self>;
                type Accesser<CS: frand_node::ext::System> = frand_node::ext::terminal::Accesser<Self, CS>;
                type Node<'n, CS: frand_node::ext::System> = frand_node::ext::terminal::Node<'n, Self, CS>;

                fn from_payload(payload: &frand_node::ext::Payload) -> Self {
                    payload.to_state()
                }

                fn to_payload(&self) -> frand_node::ext::Payload {
                    frand_node::ext::Payload::from_state(self)
                }

                fn into_message(self) -> Self::Message {
                    self
                }
            }
        )*      
    };
}

#[macro_export]
macro_rules! impl_terminal_message_for {
    ( $($tys: ty),+ $(,)? ) => {   
        $(
            impl frand_node::ext::Message for $tys {       
                type State = Self;

                fn from_packet(
                    packet: &frand_node::ext::Packet,
                    _parent_key: Key,
                    _depth: usize,                 
                ) -> Result<Self> {
                    Ok(packet.payload().to_state())
                }     

                fn to_packet(
                    &self, 
                    key: frand_node::ext::Key,
                ) -> frand_node::ext::Packet {
                    frand_node::ext::Packet::new(key, self.to_payload())
                }

                fn apply_to(&self, state: &mut Self::State) {
                    *state = *self;
                }
            }
        )*      
    };
}

#[macro_export]
macro_rules! impl_terminal_for {
    ( $($tys: ty),+ $(,)? ) => {   
        $(
            impl_terminal_state_for!{ $tys }
            impl_terminal_message_for!{ $tys }
            impl frand_node::ext::Fallback for $tys { 
                #[allow(unused_variables)]
                fn fallback<CS: frand_node::ext::System>(
                    node: Self::Node<'_, CS>, 
                    message: Self::Message, 
                    delta: Option<std::time::Duration>,
                ) {}
            }
            impl frand_node::ext::System for $tys { }
        )*      
    };
}

impl_terminal_for!{ 
    i8, i16, i32, i64, i128, 
    u8, u16, u32, u64, u128, 
    f32, f64,
    char, bool, (),
}