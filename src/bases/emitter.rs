use std::{fmt::Debug, future::Future};
use super::{AnchorKey, Packet};

pub trait Emitter {
    fn emit<E: 'static + Emitable>(&self, emitable: E);
    fn emit_future<Fu, E>(&self, future: Fu)    
    where 
    Fu: 'static + Future<Output = E> + Send,
    E: 'static + Emitable + Sized;
}

pub trait Emitable: Debug + Send + Sync {
    fn to_packet(&self, anchor_key: &AnchorKey) -> Packet;
    fn into_packet(self, anchor_key: &AnchorKey) -> Packet;
}

impl Emitable for Packet {
    fn to_packet(&self, _anchor_key: &AnchorKey) -> Packet { self.to_owned() }
    fn into_packet(self, _anchor_key: &AnchorKey) -> Packet { self }
}