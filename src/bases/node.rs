use std::future::Future;
use super::*;

pub trait Node<S: State>: System {   
    fn key(&self) -> &NodeKey;
    fn emitter(&self) -> Option<&Emitter>;
    fn clone_state(&self) -> S;

    fn emit(&self, state: S) {
        if let Some(emitter) = self.emitter() {
            emitter.emit(self.key().clone(), state)
        }
    }

    fn emit_carry(&self, state: S) {
        if let Some(emitter) = self.emitter() {
            emitter.emit_carry(self.key().clone(), state)
        }
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        if let Some(emitter) = self.emitter() {
            emitter.emit_future(self.key().clone(), future)
        }
    }
}

pub trait NewNode<S: State> {
    fn new(
        key: Vec<NodeId>,
        id: Option<NodeId>,
        emitter: Option<&Emitter>,
    ) -> Self;
}