use std::{future::Future, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use crate::bases::*;

#[derive(Debug, Clone)]
pub struct TerminalConsensus<S: State> {
    key: NodeKey,
    state: Arc<RwLock<S>>,    
}

#[derive(Debug, Clone)]
pub struct TerminalNode<S: State> {
    key: NodeKey,
    reporter: Reporter,
    state: Arc<RwLock<S>>,    
}

impl<S: State> TerminalConsensus<S> {
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

impl<S> Default for TerminalConsensus<S> 
where S: State<Message = S, Node = TerminalNode<S>, Consensus = Self> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<S> Consensus<S> for TerminalConsensus<S> 
where S: State<Message = S, Node = TerminalNode<S>, Consensus = Self> {      
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self {
        if let Some(id) = id { key.push(id); }
        
        Self { 
            key: key.into_boxed_slice(),   
            state: Default::default(),
        }
    }
    
    fn new_node(&self, reporter: &Reporter) -> TerminalNode<S> {
        Node::new_from(self, reporter)
    }

    fn clone_state(&self) -> S {
        self.read().clone()
    }
    
    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<(), PacketError> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => Ok(*self.write() = packet.read_state()),
        }
    }
    
    fn apply_state(&mut self, state: S) {
        *self.write() = state;
    }
    
    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<S::Message, PacketError> {
        match packet.get_id(depth) {
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => {
                let state: S = packet.read_state();    
                *self.write() = state.clone();                
                Ok(state)
            },
        }
    }
}

impl<S: State> TerminalNode<S> {
    pub fn v(&self) -> RwLockReadGuard<S> { self.read() }

    fn read(&self) -> RwLockReadGuard<S> { 
        self.state.read()        
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }
}

impl<S> Node<S> for TerminalNode<S> 
where S: State<Consensus = TerminalConsensus<S>> {    
    type State = S;
    
    fn new_from(
        consensus: &TerminalConsensus<S>,
        reporter: &Reporter,
    ) -> Self {
        Self { 
            key: consensus.key.clone(), 
            reporter: reporter.clone(), 
            state: consensus.state.clone(), 
        }
    }

    fn clone_state(&self) -> S {
        self.read().clone()
    }
}

impl<S: State> Emitter<S> for TerminalNode<S> {    
    fn emit(&self, state: S) {
        self.reporter.report(&self.key, state)
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.reporter.report_future(&self.key, future)
    }
}