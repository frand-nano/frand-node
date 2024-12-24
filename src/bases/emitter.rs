use std::{fmt::Debug, future::Future};
use super::*;

pub trait Emitter<S: State>: Clone + Sized + Send + Sync {
    fn emit(&self, emitable: S);
    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send;
}

pub trait Emitable: 'static + Debug + Send + Sync {
    fn to_packet(&self, node_key: &NodeKey) -> Packet;
    fn into_packet(self, node_key: &NodeKey) -> Packet;
}

impl Emitable for Packet {
    fn to_packet(&self, _node_key: &NodeKey) -> Packet { self.to_owned() }
    fn into_packet(self, _node_key: &NodeKey) -> Packet { self }
}