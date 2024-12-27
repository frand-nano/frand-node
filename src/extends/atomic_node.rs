use std::{future::Future, marker::PhantomData, sync::{atomic::*, Arc}};
use crate::bases::*;

#[derive(Debug, Clone)]
pub struct AtomicConsensus<M: Message, S: AtomicState> {
    key: NodeKey,
    state: S::Atomic,   
    _phantom: PhantomData<M>, 
}

#[derive(Debug, Clone)]
pub struct AtomicNode<M: Message, S: AtomicState> {
    key: NodeKey,
    reporter: Reporter<M>,
    state: S::Atomic,   
}

impl<M: Message, S: AtomicState> AtomicConsensus<M, S> {
    #[inline] pub fn v(&self) -> S { self.state.load(Ordering::Relaxed) }
    #[inline] fn set_v(&self, state: S) { self.state.store(state, Ordering::Relaxed) }
}

impl<M: Message, S> Default for AtomicConsensus<M, S> 
where S: AtomicState<Message = S, Node<M> = AtomicNode<M, S>, Consensus<M> = Self> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<M: Message, S> Consensus<M, S> for AtomicConsensus<M, S> 
where S: AtomicState<Message = S, Node<M> = AtomicNode<M, S>, Consensus<M> = Self> {      
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self {
        if let Some(id) = id { key.push(id); }
        
        Self { 
            key: key.into_boxed_slice(),   
            state: Atomic::new(Default::default()),
            _phantom: Default::default(),
        }
    }
    
    fn new_node(&self, reporter: &Reporter<M>) -> AtomicNode<M, S> {
        Node::new_from(self, reporter)
    }

    #[inline]
    fn clone_state(&self) -> S { 
        self.v() 
    }
    
    #[inline]
    fn apply(&mut self, message: S::Message) {
        self.apply_state(message)
    }

    #[inline]
    fn apply_state(&mut self, state: S) {
        self.set_v(state);
    }
}

impl<M: Message, S: AtomicState> AtomicNode<M, S> {
    #[inline] pub fn v(&self) -> S { self.state.load(Ordering::Relaxed) }
}

impl<M: Message, S> Node<M, S> for AtomicNode<M, S> 
where S: AtomicState<Consensus<M> = AtomicConsensus<M, S>> {    
    type State = S;
    
    fn new_from(
        consensus: &AtomicConsensus<M, S>,
        reporter: &Reporter<M>,
    ) -> Self {
        Self { 
            key: consensus.key.clone(), 
            reporter: reporter.clone(), 
            state: consensus.state.clone(), 
        }
    }

    #[inline]
    fn clone_state(&self) -> S {
        self.v()
    }
}

impl<M: Message, S: AtomicState> Emitter<M, S> for AtomicNode<M, S> {    
    fn emit(&self, state: S) {
        self.reporter.report(&self.key, state)
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.reporter.report_future(self.key.clone(), future)
    }
}

macro_rules! impl_atomic_state_for {
    ( $($tys: ty : $atomics: ty : $arc_atomics: ident),+ $(,)? ) => {   
        $(
            impl State for $tys {
                type Message = Self;
                type Consensus<M: Message> = AtomicConsensus<M, Self>;
                type Node<M: Message> = AtomicNode<M, Self>;

                fn apply(
                    &mut self,  
                    message: Self::Message,
                ) {
                    *self = message;
                }
            }

            impl AtomicState for $tys {
                type Atomic = $arc_atomics;
            }

            #[derive(Debug, Clone)]
            pub struct $arc_atomics(Arc<$atomics>);

            impl Atomic<$tys> for $arc_atomics {
                #[inline] fn new(value: $tys) -> Self { Self(Arc::new(<$atomics>::new(value))) }
                #[inline] fn load(&self, ordering: Ordering) -> $tys { self.0.load(ordering) }
                #[inline] fn store(&self, value: $tys, ordering: Ordering) { self.0.store(value, ordering) }
            }
        )*      
    };
}

impl_atomic_state_for!{ 
    i8: AtomicI8: ArcAtomicI8, 
    i16: AtomicI16: ArcAtomicI16, 
    i32: AtomicI32: ArcAtomicI32, 
    i64: AtomicI64: ArcAtomicI64, 
    u8: AtomicU8: ArcAtomicU8, 
    u16: AtomicU16: ArcAtomicU16, 
    u32: AtomicU32: ArcAtomicU32, 
    u64: AtomicU64: ArcAtomicU64, 
    bool: AtomicBool: ArcAtomicBool, 
}