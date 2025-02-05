use std::fmt::Debug;
use serde::{de::DeserializeOwned, Serialize};
use crate::ext::*;

pub trait State: 'static + Debug + Default + Clone + Send + Sync + Unpin + Serialize + DeserializeOwned {
    const NODE_SIZE: IdSize;
    const NODE_ALT_SIZE: AltSize;

    type Message: Message<State = Self>;
    type Emitter: Emitter<Self>;
    type Accesser<CS: System>: Accesser<Self, CS>;
    type Node<'n, CS: System>: Node<'n, Self> + NewNode<'n, Self, CS>;

    fn from_payload(payload: &Payload) -> Self;    
    fn to_payload(&self) -> Payload;

    fn into_message(self) -> Self::Message;
}