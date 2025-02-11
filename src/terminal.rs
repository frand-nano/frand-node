use crate::ext::*;
use crate::frand_node;

pub mod terminal {
    pub use super::*;

    #[derive(Debug, Clone)]
    pub struct Emitter<S: System> {
        callback: Callback<S>,
    }

    #[derive(Debug, Clone)]
    pub struct Accesser<S: System> {
        lookup: Lookup<S>,
    }
    
    #[derive(Debug, Clone)]
    pub struct Node<'n, S: System> {
        accesser: &'n S::Accesser,
        emitter: &'n S::Emitter,
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
        
    impl<S: System> super::Accesser<S> for Accesser<S> {
        fn lookup(&self) -> &Lookup<S> {
            &self.lookup
        }

        fn new<CS: System>(builder: LookupBuilder<CS, S>) -> Self {
            Self { 
                lookup: builder.build(|state| state.cloned()), 
            }
        }
    }
    
    impl<'n, S: System> super::Node<'n, S> for Node<'n, S> {
        fn accesser(&self) -> &S::Accesser { self.accesser }
        fn emitter(&self) -> &S::Emitter { self.emitter }
        fn transient(&self) -> &Transient { &self.transient }
    }
    
    impl<'n, S: System> NewNode<'n, S> for Node<'n, S> {
        fn new(
            accesser: &'n S::Accesser,
            emitter: &'n S::Emitter,
            transient: &'n Transient,        
        ) -> Self {
            Self { 
                accesser,
                emitter,
                transient,
            }
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
                type Accesser = frand_node::ext::terminal::Accesser<Self>;
                type Node<'n> = frand_node::ext::terminal::Node<'n, Self>;

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
                fn fallback(
                    node: Self::Node<'_>, 
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