use std::{fmt::Debug, future::Future, ops::Deref, sync::{Arc, RwLockReadGuard}};
use crate::ext::*;

pub trait Node<'n, S: State>: Debug + Deref<Target = S> {
    fn alt(&self) -> &Alt;
    fn emitter(&self) -> &S::Emitter;

    fn consist(&self) -> &Consist { self.emitter().callback().consist() }

    fn emit(&self, state: S) {
        Emitter::emit(self.emitter(), self.alt(), state);
    }

    fn emit_carry(&self, state: S) {
        Emitter::emit_carry(self.emitter(), self.alt(), state);
    }

    fn emit_future<F>(&self, future: F) 
    where F: Future<Output = S::Message> + 'static + Send + Sync {
        Emitter::emit_future(self.emitter(), self.alt(), future);
    }
}

pub trait NewNode<'n, S: State, CS: System> {
    fn new(
        emitter: &'n S::Emitter,
        accesser: &'n S::Accesser<CS>,
        consensus: &'n Arc<RwLockReadGuard<'n, CS>>,
        alt: &'n Alt,        
    ) -> Self;

    fn new_alt(
        &self,
        alt: Alt,   
    ) -> ConsensusRead<'n, S, CS>;
}