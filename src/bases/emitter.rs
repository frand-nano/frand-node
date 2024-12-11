use std::fmt::Debug;
use serde::{de::DeserializeOwned, Serialize};
use super::{AnchorKey, Packet, State};

pub trait Emitter<S: State> {
    fn emit(&self, state: S);
}

pub trait Emitable: Debug + Serialize + DeserializeOwned {
    fn into_packet(self, anchor_key: AnchorKey) -> Packet;
}

impl Emitable for Packet {
    fn into_packet(self, _anchor_key: AnchorKey) -> Packet { self }
}