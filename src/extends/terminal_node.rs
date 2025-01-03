use std::{future::Future, marker::PhantomData, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use crate::bases::*;

#[derive(Debug, Clone)]
pub struct TerminalConsensus<M: Message, S: State> {
    key: NodeKey,
    state: Arc<RwLock<S>>,    
    _phantom: PhantomData<M>,
}

#[derive(Debug, Clone)]
pub struct TerminalNode<M: Message, S: State> {
    key: NodeKey,
    callback: Callback<M>,
    future_callback: FutureCallback<M>,
    state: Arc<RwLock<S>>,    
}

impl<M: Message, S: State> TerminalConsensus<M, S> {
    pub fn v(&self) -> RwLockReadGuard<S> { self.read() }

    fn read(&self) -> RwLockReadGuard<S> { 
        self.state.read()        
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }

    fn write(&mut self) -> RwLockWriteGuard<S> { 
        self.state.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }
}

impl<M: Message, S> Default for TerminalConsensus<M, S> 
where S: State<Message = S, Node<M> = TerminalNode<M, S>, Consensus<M> = Self> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<M: Message, S> Consensus<M, S> for TerminalConsensus<M, S> 
where S: State<Message = S, Node<M> = TerminalNode<M, S>, Consensus<M> = Self> {      
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self {
        if let Some(id) = id { key.push(id); }
        
        Self { 
            key: key.into_boxed_slice(),   
            state: Default::default(),
            _phantom: Default::default(),
        }
    }
    
    fn new_node(
        &self, 
        callback: &Callback<M>, 
        future_callback: &FutureCallback<M>,
    ) -> TerminalNode<M, S> {
        Node::new_from(self, callback, future_callback)
    }

    fn clone_state(&self) -> S {
        self.read().clone()
    }
    
    fn apply(&mut self, message: S::Message) {
        self.apply_state(message)
    }
    
    fn apply_state(&mut self, state: S) {
        *self.write() = state;
    }
}

impl<M: Message, S: State> TerminalNode<M, S> {
    pub fn v(&self) -> RwLockReadGuard<S> { self.read() }

    fn read(&self) -> RwLockReadGuard<S> { 
        self.state.read()        
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }
}

impl<M: Message, S> Node<M, S> for TerminalNode<M, S> 
where S: State<Consensus<M> = TerminalConsensus<M, S>> {    
    type State = S;
    
    fn key(&self) -> &NodeKey { &self.key }

    fn new_from(
        consensus: &TerminalConsensus<M, S>,
        callback: &Callback<M>,
        future_callback: &FutureCallback<M>,
    ) -> Self {
        Self { 
            key: consensus.key.clone(), 
            callback: callback.clone(), 
            future_callback: future_callback.clone(), 
            state: consensus.state.clone(), 
        }
    }

    fn clone_state(&self) -> S {
        self.read().clone()
    }
}

impl<M: Message, S: State> Emitter<M, S> for TerminalNode<M, S> {    
    fn emit(&self, state: S) {
        self.callback.emit(self.key.clone(), state)
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.future_callback.emit(self.key.clone(), future)
    }
}