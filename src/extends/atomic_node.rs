use std::{future::Future, sync::{atomic::*, Arc}};
use crate::bases::*;

#[derive(Debug, Clone)]
pub struct AtomicConsensus<S: AtomicState> {
    key: NodeKey,
    state: S::Atomic,    
}

#[derive(Debug, Clone)]
pub struct AtomicNode<S: AtomicState> {
    key: NodeKey,
    reporter: Reporter,
    state: S::Atomic,   
}

impl<S: AtomicState> AtomicConsensus<S> {
    #[inline] pub fn v(&self) -> S { self.state.load(Ordering::Relaxed) }
    #[inline] fn set_v(&self, state: S) { self.state.store(state, Ordering::Relaxed) }
}

impl<S> Default for AtomicConsensus<S> 
where S: AtomicState<Message = S, Node = AtomicNode<S>, Consensus = Self> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<S> Consensus<S> for AtomicConsensus<S> 
where S: AtomicState<Message = S, Node = AtomicNode<S>, Consensus = Self> {      
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self {
        if let Some(id) = id { key.push(id); }
        
        Self { 
            key: key.into_boxed_slice(),   
            state: S::Atomic::new(Default::default()),
        }
    }
    
    fn new_node(&self, reporter: &Reporter) -> AtomicNode<S> {
        Node::new_from(self, reporter)
    }

    #[inline]
    fn clone_state(&self) -> S { 
        self.v() 
    }
    
    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<(), PacketError> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => Ok(self.set_v(packet.read_state())),
        }
    }

    #[inline]
    fn apply_state(&mut self, state: S) {
        self.set_v(state);
    }
    
    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message, PacketError> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => {
                let state: S = packet.read_state();    
                self.set_v(state);                
                Ok(state)
            },
        }
    }
}

impl<S: AtomicState> AtomicNode<S> {
    #[inline] pub fn v(&self) -> S { self.state.load(Ordering::Relaxed) }
}

impl<S> Node<S> for AtomicNode<S> 
where S: AtomicState<Consensus = AtomicConsensus<S>> {    
    type State = S;
    
    fn new_from(
        consensus: &AtomicConsensus<S>,
        reporter: &Reporter,
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

impl<S: AtomicState> Emitter<S> for AtomicNode<S> {    
    fn emit(&self, state: S) {
        self.reporter.report(&self.key, state)
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.reporter.report_future(&self.key, future)
    }
}

macro_rules! impl_atomic_state_for {
    ( $($tys: ty : $atomics: ty : $arc_atomics: ident),+ $(,)? ) => {   
        $(
            impl State for $tys {
                type Message = Self;
                type Consensus = AtomicConsensus<Self>;
                type Node = AtomicNode<Self>;

                fn apply(
                    &mut self, 
                    depth: usize, 
                    packet: Packet,
                ) -> core::result::Result<(), PacketError>  {
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