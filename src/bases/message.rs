use anyhow::Result;
use super::Packet;

pub trait Message: Sized {
    fn from_packet(depth: usize, packet: &Packet) -> Result<Self>;    
}