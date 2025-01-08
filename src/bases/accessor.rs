use std::fmt::Debug;
use super::*;

pub trait Accessor: 'static + Debug + Default + Clone + Sized + Send + Sync {
    type State: State<State = Self::State, Message = Self::Message, Node = Self::Node>;
    type Message: Message;
    type Node: Node<Self::State, State = Self::State, Message = Self::Message, Node = Self::Node> + NewNode<Self::State>;
}