use std::{ops::Deref, sync::{Arc, RwLockReadGuard}};
use crate::ext::*;

#[derive(Debug)]
pub struct ConsensusRead<'n, S: State, CS: System> {
    emitter: &'n S::Emitter,
    accesser: &'n S::Accesser<CS>,
    consensus: Arc<RwLockReadGuard<'n, CS>>,
    alt: Alt,
}

impl<'n, S: State, CS: System> ConsensusRead<'n, S, CS> {
    pub fn new(
        emitter: &'n S::Emitter,
        accesser: &'n S::Accesser<CS>,
        consensus: Arc<RwLockReadGuard<'n, CS>>,
        alt: Alt,
    ) -> Self {     
        Self { 
            emitter, 
            accesser, 
            consensus, 
            alt, 
        }
    }

    pub fn node(&'n self) -> S::Node<'n, CS> 
    where S::Node<'n, CS>: NewNode<'n, S, CS> {
        NewNode::new(
            self.emitter,
            self.accesser,
            &self.consensus,
            &self.alt,
        )
    }
}

impl<'n, S: System, CS: System> Deref for ConsensusRead<'n, S, CS> {
    type Target = S;
    fn deref(&self) -> &Self::Target { 
        (self.accesser)(&self.consensus, self.alt)
    }
}

impl<'n, S: System, CS: System> Node<'n, S> for ConsensusRead<'n, S, CS> {
    fn alt(&self) -> &Alt { &self.alt }
    fn emitter(&self) -> &<S as State>::Emitter { &self.emitter }
}