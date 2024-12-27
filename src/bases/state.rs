use std::{any::Any, fmt::Debug};
use serde::{de::DeserializeOwned, Serialize};

use super::*;

pub trait State: 'static + Default + Clone + Debug + Send + Sync + Sized + Any + Serialize + DeserializeOwned {
    type Message: Message;
    type Consensus<M: Message>: Consensus<M, Self>;
    type Node<M: Message>: Node<M, Self>;

    fn apply(&mut self, message: Self::Message);    
}