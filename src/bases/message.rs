use std::fmt::Debug;
use super::*;

pub trait Message: 'static + Debug + Clone + Sized + Send + Sync {   
    fn from_packet_message(
        packet: &PacketMessage, 
        depth: usize, 
    ) -> Result<Self, MessageError>;

    fn from_packet(
        packet: &Packet, 
        depth: usize, 
    ) -> Result<Self, PacketError>;

    fn to_packet(
        &self,
        header: &Header, 
    ) -> Result<Packet, MessageError>;
}