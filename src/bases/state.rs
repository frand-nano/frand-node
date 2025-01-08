use serde::{de::DeserializeOwned, Serialize};
use super::*;

pub trait State: Serialize + DeserializeOwned + Accessor<State = Self> + Emitable {
    const NODE_SIZE: Index;

    fn apply(&mut self, message: Self::Message);    

    unsafe fn from_emitable(emitable: &Box<dyn Emitable>) -> Self {
        (emitable.as_ref() as *const dyn Emitable)
        .cast::<Self>()
        .as_ref().cloned().unwrap()
    }
}