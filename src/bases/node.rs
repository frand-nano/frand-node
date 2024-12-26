use std::{future::Future, sync::Arc};
use super::*;

pub trait Node<S: State>: Emitter<S> {
    type State: State;

    fn new_from(
        consensus: &S::Consensus,
        reporter: &Reporter,
    ) -> Self;

    fn clone_state(&self) -> S;
}

impl<S: State, N: Node<S>> Node<S> for Arc<N> {
    type State = S;

    fn new_from(
        consensus: &<S as State>::Consensus,
        reporter: &Reporter,
    ) -> Self {
        Arc::new(N::new_from(consensus, reporter))
    }

    fn clone_state(&self) -> S {
        self.as_ref().clone_state()
    }
}

impl<S: State, N: Node<S>> Emitter<S> for Arc<N> {
    fn emit(&self, emitable: S) {
        self.as_ref().emit(emitable)
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.as_ref().emit_future(future)
    }
}