use super::*;

pub trait Message: Clone + Sized + Send + Sync {
    fn from_packet(depth: usize, packet: &Packet) -> Result<Self, PacketError>;    
}