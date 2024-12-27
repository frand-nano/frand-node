use super::*;

pub trait Consensus<M: Message, S: State>: Sized + Clone + Default + Send + Sync {   
    fn new(
        key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self;

    fn new_node(&self, reporter: &Reporter<M>) -> S::Node<M>;
    fn clone_state(&self) -> S;
    fn apply(&mut self, message: S::Message);   
    fn apply_state(&mut self, state: S);
}