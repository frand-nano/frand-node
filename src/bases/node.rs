use std::{fmt::Debug, future::Future, ops::Deref, sync::{Arc, RwLockReadGuard}};
use crate::ext::*;

pub trait Node<'n, S: State>: Debug + Deref<Target = S> {
    fn transient(&self) -> &Transient;
    fn emitter(&self) -> &S::Emitter;

    fn consist(&self) -> &Consist { self.emitter().callback().consist() }

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

pub trait NewNode<'n, S: State, CS: System> {
    fn new(
        emitter: &'n S::Emitter,
        accesser: &'n S::Accesser<CS>,
        consensus: &'n Arc<RwLockReadGuard<'n, CS>>,
        transient: &'n Transient,        
    ) -> Self;

    fn alt(
        &self,
        transient: Transient,   
    ) -> ConsensusRead<'n, S, CS>;
}