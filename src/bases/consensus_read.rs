use std::{ops::Deref, sync::{Arc, RwLockReadGuard}};
use crate::ext::*;

#[derive(Debug)]
pub struct ConsensusRead<'n, S: State, CS: System> {
    emitter: &'n S::Emitter,
    accesser: &'n S::Accesser<CS>,
    consensus: Arc<RwLockReadGuard<'n, CS>>,
    transient: Transient,
}

impl<'n, S: State, CS: System> ConsensusRead<'n, S, CS> {
    pub fn new(
        emitter: &'n S::Emitter,
        accesser: &'n S::Accesser<CS>,
        consensus: Arc<RwLockReadGuard<'n, CS>>,
        transient: Transient,
    ) -> Self {     
        Self { 
            emitter, 
            accesser, 
            consensus, 
            transient, 
        }
    }

    pub fn node(&'n self) -> S::Node<'n, CS> 
    where S::Node<'n, CS>: NewNode<'n, S, CS> {
        NewNode::new(
            self.emitter,
            self.accesser,
            &self.consensus,
            &self.transient,
        )
    }
}

impl<'n, S: System, CS: System> Deref for ConsensusRead<'n, S, CS> {
    type Target = S;
    fn deref(&self) -> &Self::Target { 
        (self.accesser)(&self.consensus, self.transient)
    }
}

impl<'n, S: System, CS: System> Node<'n, S> for ConsensusRead<'n, S, CS> {
    fn transient(&self) -> &Transient { &self.transient }
    fn emitter(&self) -> &<S as State>::Emitter { &self.emitter }
}