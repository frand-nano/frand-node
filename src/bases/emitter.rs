use anyhow::Result;
use std::fmt::Debug;
use serde::{de::DeserializeOwned, Serialize};
use super::{NodeKey, Packet, State};

pub trait Emitter<S: State> {
    fn emit(&self, state: S) -> Result<()>;
}

pub trait Emitable: Debug + Serialize + DeserializeOwned {
    fn into_packet(self, node_key: NodeKey) -> Packet;
}

impl Emitable for Packet {
    fn into_packet(self, _node_key: NodeKey) -> Packet { self }
}