use std::fmt::Display;
use crate::ext::*;

pub type Result<T, E = PacketError> = core::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct PacketError {
    packet: Packet,
    id_delta: Option<IdDelta>,
    depth: Option<usize>,
    message: String,
}

impl PacketError {
    pub fn new(
        packet: Packet,
        id_delta: Option<IdDelta>,
        depth: Option<usize>,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            packet,
            id_delta,
            depth,
            message: message.as_ref().to_string(),
        }
    }
}

impl Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "{} id_delta:{:?} depth:{:?} packet:{:?}", 
            self.message, self.id_delta, self.depth, self.packet,
        )
    }
}

impl core::error::Error for PacketError {}