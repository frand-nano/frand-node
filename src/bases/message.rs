use anyhow::Result;
use super::Packet;

pub trait Message: Clone + Sized + Send + Sync {
    fn from_packet(depth: usize, packet: &Packet) -> Result<Self>;    
}