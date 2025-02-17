use std::{fmt::Debug, future::Future};
use crate::ext::*;

pub trait Node<'n, S: State>: Debug + Clone {
    fn accesser(&self) -> &S::Accesser;
    fn emitter(&self) -> &S::Emitter;
    fn transient(&self) -> &Transient;
    fn callback_mode(&self) -> &CallbackMode;

    fn consist(&self) -> &Consist {
        self.emitter().callback().consist()
    }

    fn key(&self) -> Key {
        Key::new(*self.consist(), *self.transient())
    }

    fn alt(
        &self, 
        parent_consist: &Consist, 
        index: AltIndex,
    ) -> NodeAlt<'_, S> {
        NodeAlt {
            accesser: self.accesser(),
            emitter: self.emitter(),
            transient: self.transient().alt(parent_consist.alt_depth(), index),
            callback_mode: self.callback_mode(),
        }        
    }

    fn v(&self) -> S where S: Copy { 
        self.accesser().lookup().get(self.transient()).unwrap_or_default() 
    }

    fn clone_state(&self) -> Option<S> { 
        self.accesser().lookup().get(self.transient())
    }

    fn lookup(&self) -> impl Fn() -> Option<S> + 'static { 
        let lookup = self.accesser().lookup().clone();
        let transient = *self.transient();
        move || lookup.get(&transient)
    }

    fn emit(&self, state: S) {
        Emitter::emit(
            self.emitter(), 
            self.callback_mode(), 
            self.transient(), 
            state,
        );
    }

    fn emit_carry<F>(&self, lookup: F) 
    where F: Fn() -> S::Message + 'static + Send + Sync {
        Emitter::emit_carry(
            self.emitter(), 
            self.callback_mode(), 
            self.transient(), 
            lookup,
        );
    }

    fn emit_future<F>(&self, future: F) 
    where F: Future<Output = S::Message> + 'static + Send + Sync {
        Emitter::emit_future(
            self.emitter(), 
            self.callback_mode(), 
            self.transient(), 
            future,
        );
    }
}

pub trait NewNode<'n, S: State> {
    fn new(
        accesser: &'n S::Accesser,
        emitter: &'n S::Emitter,
        callback_mode: &'n CallbackMode,
        transient: &'n Transient,        
    ) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct NodeAlt<'n, S: State> {
    accesser: &'n S::Accesser,
    emitter: &'n S::Emitter,
    callback_mode: &'n CallbackMode,
    transient: Transient,    
}

impl<'n, S: State> Node<'n, S> for NodeAlt<'n, S> {
    fn accesser(&self) -> &S::Accesser { self.accesser }
    fn emitter(&self) -> &S::Emitter { self.emitter }
    fn callback_mode(&self) -> &CallbackMode { self.callback_mode }
    fn transient(&self) -> &Transient { &self.transient }
}

impl<'n, S: State> NodeAlt<'n, S> {
    pub fn node(&'n self) -> S::Node<'n> {
        NewNode::new(
            self.accesser,
            self.emitter,
            self.callback_mode,
            &self.transient,
        )
    }
}