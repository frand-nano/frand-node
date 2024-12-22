use super::*;

pub trait Node<S: State>: Emitter<S> {
    fn new_from(
        consensus: &S::Consensus,
        reporter: &Reporter,
    ) -> Self;

    fn clone_state(&self) -> S;
}