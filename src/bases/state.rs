use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use crate::ext::*;

pub trait State: 'static + Debug + Default + Clone + Send + Sync + Unpin + Serialize + for<'de> Deserialize<'de> {
    const NODE_SIZE: IdSize;
    const NODE_ALT_SIZE: AltSize;

    type Message: Message<State = Self>;
    type Emitter: Emitter<Self>;
    type Accesser: Accesser<Self>;
    type Node<'n>: Node<'n, Self> + NewNode<'n, Self>;

    fn from_payload(payload: &Payload) -> Self;    
    fn to_payload(&self) -> Payload;

    fn into_message(self) -> Self::Message;
}