use std::{marker::PhantomData, sync::{atomic::*, Arc}};
use crate::bases::*;

#[derive(Debug, Clone)]
pub struct AtomicNode<S: State, A: AtomicState<S>> {
    _phantom: PhantomData<S>,
    key: Key,
    emitter: Option<Emitter>,
    state: A,   
}

impl<S: State, A: AtomicState<S>> AtomicNode<S, A> {
    #[inline] pub fn v(&self) -> S { self.state.load(Ordering::Relaxed) }
}

impl<S: State + Message, A: AtomicState<S>> Default for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {
    fn default() -> Self { Self::new(0.into(), 0, None) }
}

impl<S: State + Message, A: AtomicState<S>> Accessor for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {
    type State = S;
    type Message = S;     
    type Node = Self;
}

impl<S: State + Message, A: AtomicState<S>> Fallback for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {
    fn fallback(&self, _message: Self::Message, _delta: Option<f32>) {}
}

impl<S: State + Message, A: AtomicState<S>> System for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {
    fn handle(&self, _message: Self::Message, _delta: Option<f32>) {}
}

impl<S: State + Message, A: AtomicState<S>> Node<S> for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {  
    fn key(&self) -> Key { self.key }
    fn emitter(&self) -> Option<&Emitter> { self.emitter.as_ref() }
    fn clone_state(&self) -> S { self.v() }    
}

impl<S: State + Message, A: AtomicState<S>> NewNode<S> for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {  
    fn new(
        mut key: Key,
        index: Index,
        emitter: Option<&Emitter>,
    ) -> Self {
        key = key + index;
        
        Self { 
            _phantom: Default::default(),
            key, 
            emitter: emitter.cloned(),
            state: AtomicState::new(Default::default()),
        }
    }
}

impl<S: State + Message, A: AtomicState<S>> Consensus<S> for AtomicNode<S, A> 
where S: State<Message = S, Node = Self> {  
    fn new_from(
        node: &Self,
        emitter: Option<&Emitter>,
    ) -> Self {
        Self {
            _phantom: Default::default(),
            key: node.key,
            emitter: emitter.cloned(),
            state: node.state.clone(),
        }
    }

    fn set_emitter(&mut self, emitter: Option<&Emitter>) { self.emitter = emitter.cloned() }
    fn apply(&self, message: S::Message) { self.state.store(message, Ordering::Relaxed) }
    fn apply_state(&self, state: S) { self.state.store(state, Ordering::Relaxed) }    
}

macro_rules! impl_atomic_state_for {
    ( $($tys: ty : $atomics: ty),+ $(,)? ) => {   
        $(
            impl Accessor for $tys {
                type State = Self;
                type Message = Self;
                type Node = AtomicNode<Self, Arc<$atomics>>;
            }

            impl Emitable for $tys {}

            impl State for $tys {
                const NODE_SIZE: Index = 1; 

                fn apply(
                    &mut self,  
                    message: Self::Message,
                ) {
                    *self = message;
                }
            }

            impl AtomicState<$tys> for Arc<$atomics> {
                #[inline] fn new(value: $tys) -> Self { Arc::new(<$atomics>::new(value)) }
                #[inline] fn load(&self, ordering: Ordering) -> $tys { self.as_ref().load(ordering) }
                #[inline] fn store(&self, value: $tys, ordering: Ordering) { self.as_ref().store(value, ordering) }
            }
        )*      
    };
}

impl_atomic_state_for!{ 
    i8: AtomicI8, 
    i16: AtomicI16, 
    i32: AtomicI32, 
    i64: AtomicI64, 
    u8: AtomicU8, 
    u16: AtomicU16, 
    u32: AtomicU32, 
    u64: AtomicU64, 
    bool: AtomicBool, 
}