use super::*;

pub trait Node<S: State>: AsRef<Self> + Emitter<S> {
    type State: State;

    fn new_from(
        consensus: &S::Consensus,
        reporter: &Reporter,
    ) -> Self;

    fn clone_state(&self) -> S;
}