use std::ops::Deref;
use crate::bases::Packet;

pub type PacketSender = crossbeam::channel::Sender<Packet>;

#[derive(Debug, Clone)]
pub struct Sender(PacketSender);

impl Deref for Sender {
    type Target = PacketSender;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<PacketSender> for Sender {
    fn from(sender: PacketSender) -> Self {
        Self(sender)
    }
}