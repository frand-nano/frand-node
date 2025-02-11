use std::{fmt::Debug, future::Future};
use crate::ext::*;

pub trait Node<'n, S: State>: Debug + Clone {
    fn accesser(&self) -> &S::Accesser;
    fn emitter(&self) -> &S::Emitter;
    fn transient(&self) -> &Transient;

    fn consist(&self) -> &Consist {
        self.emitter().callback().consist()
    }

    fn alt(&self, parent_consist: &Consist, index: AltIndex) -> NodeAlt<'_, S> {
        NodeAlt {
            accesser: self.accesser(),
            emitter: self.emitter(),
            transient: self.transient().alt(parent_consist.alt_depth(), index),
        }        
    }

    fn clone_state(&self) -> Option<S> { 
        (self.accesser().lookup())(self.transient()) 
    }

    fn v(&self) -> S where S: Copy { 
        (self.accesser().lookup())(self.transient()).unwrap_or_default() 
    }

    fn emit(&self, state: S) {
        Emitter::emit(self.emitter(), self.transient(), state);
    }

    fn emit_carry(&self, state: S) {
        Emitter::emit_carry(self.emitter(), self.transient(), state);
    }

    fn emit_future<F>(&self, future: F) 
    where F: Future<Output = S::Message> + 'static + Send + Sync {
        Emitter::emit_future(self.emitter(), self.transient(), future);
    }
}

pub trait NewNode<'n, S: State> {
    fn new(
        accesser: &'n S::Accesser,
        emitter: &'n S::Emitter,
        transient: &'n Transient,        
    ) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct NodeAlt<'n, S: State> {
    accesser: &'n S::Accesser,
    emitter: &'n S::Emitter,
    transient: Transient,    
}

impl<'n, S: State> Node<'n, S> for NodeAlt<'n, S> {
    fn accesser(&self) -> &S::Accesser { self.accesser }
    fn emitter(&self) -> &S::Emitter { self.emitter }
    fn transient(&self) -> &Transient { &self.transient }
}

impl<'n, S: System + Copy> std::ops::Deref for NodeAlt<'n, S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        todo!()
    }
}

impl<'n, S: State> NodeAlt<'n, S> {
    pub fn node(&'n self) -> S::Node<'n> {
        NewNode::new(
            self.accesser,
            self.emitter,
            &self.transient,
        )
    }
}