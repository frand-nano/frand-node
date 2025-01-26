use std::fmt::Debug;
use super::*;

pub trait Message: 'static + Debug + Clone + Sized + Send + Sync {   
    fn from_packet_message(
        parent_key: Key,
        depth: Depth,
        packet: &PacketMessage, 
    ) -> Result<Self, MessageError>;

    fn from_packet(
        parent_key: Key,
        depth: Depth,
        packet: &Packet, 
    ) -> Result<Self, PacketError>;

    fn to_packet(
        &self,
        key: Key, 
    ) -> Result<Packet, MessageError>;
}