use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::bases::*;

#[derive(Debug, Clone)]
pub struct TerminalNode<S: State> {
    key: Key,
    emitter: Option<Emitter>,
    state: Arc<RwLock<S>>,    
}

impl<S: State> TerminalNode<S> {
    pub fn v(&self) -> RwLockReadGuard<S> { self.read() }

    fn read(&self) -> RwLockReadGuard<S> { 
        self.state.read()        
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }
}

impl<S: State + Message> Default for TerminalNode<S> 
where S: State<Message = S> {    
    fn default() -> Self { Self::new(0.into(), 0, None) }
}

impl<S: State + Message> Accessor for TerminalNode<S>  
where S: State<Message = S> {    
    type State = S;
    type Message = S::Message;    
    type Node = S::Node;
}

impl<S: State + Message> Fallback for TerminalNode<S> 
where S: State<Message = S> {
    fn fallback(&self, _message: Self::Message, _delta: Option<f32>) {}
}

impl<S: State + Message> System for TerminalNode<S> 
where S: State<Message = S> {
    fn handle(&self, _message: Self::Message, _delta: Option<f32>) {}
}

impl<S: State + Message> Node<S> for TerminalNode<S> 
where S: State<Message = S> {    
    fn key(&self) -> Key { self.key }
    fn emitter(&self) -> Option<&Emitter> { self.emitter.as_ref() }
    fn clone_state(&self) -> S { self.read().clone() }
}

impl<S: State + Message> NewNode<S> for TerminalNode<S> 
where S: State<Message = S> {    
    fn new(
        mut key: Key,
        index: Index,
        emitter: Option<&Emitter>,
    ) -> Self {
        key = key + index;
        
        Self { 
            key,   
            emitter: emitter.cloned(),
            state: Default::default(),
        }
    }
}

impl<S: State + Message> Consensus<S> for TerminalNode<S> 
where S: State<Message = S> {    
    fn new_from(
        node: &Self,
        emitter: Option<&Emitter>,
    ) -> Self {
        Self {
            key: node.key.clone(),
            emitter: emitter.cloned(),
            state: node.state.clone(),
        }
    }

    fn set_emitter(&mut self, emitter: Option<&Emitter>) { self.emitter = emitter.cloned() }
    fn apply(&self, message: S::Message) { self.apply_state(message) }    
    fn apply_state(&self, state: S) {     
        let mut state_mut = self.state.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        
        *state_mut = state;
    }
}