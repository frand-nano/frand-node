use std::{future::Future, sync::Arc};
use super::*;

pub trait Node<M: Message, S: State>: Emitter<M, S> {
    type State: State;

    fn new_from(
        consensus: &S::Consensus<M>,
        reporter: &Reporter<M>,
    ) -> Self;

    fn clone_state(&self) -> S;
}

impl<M: Message, S: State, N: Node<M, S>> Node<M, S> for Arc<N> {
    type State = S;

    fn new_from(
        consensus: &<S as State>::Consensus<M>,
        reporter: &Reporter<M>,
    ) -> Self {
        Arc::new(N::new_from(consensus, reporter))
    }

    fn clone_state(&self) -> S {
        self.as_ref().clone_state()
    }
}

impl<M: Message, S: State, N: Node<M, S>> Emitter<M, S> for Arc<N> {
    fn emit(&self, emitable: S) {
        self.as_ref().emit(emitable)
    }

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.as_ref().emit_future(future)
    }
}