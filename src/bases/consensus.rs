use super::*;

pub trait Consensus<S: State>: Node<S> {   
    fn new_from(
        node: &Self,
        emitter: Option<Emitter>,
    ) -> Self;

    fn set_emitter(&mut self, emitter: Option<Emitter>);
    fn apply(&self, message: S::Message);   
    fn apply_state(&self, state: S);
}