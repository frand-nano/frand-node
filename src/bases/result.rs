use core::error;
use std::fmt::{Debug, Display};
use super::*;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum Error {
    Text(String),
    Packet(PacketError),
    CrossbeamSend(crossbeam::channel::SendError<Packet>),
    TokioSend(tokio::sync::mpsc::error::SendError<Packet>),
}

#[derive(Debug, Clone)]
pub struct PacketError {
    packet: Packet,
    depth: usize,
    message: String,
}

impl PacketError {
    pub fn new(
        packet: Packet,
        depth: usize,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            packet,
            depth,
            message: message.as_ref().to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {    
        match self {
            Error::Text(err) => write!(f, "Error::Text({err})"),
            Error::Packet(err) => write!(f, "Error::Packet({err})"),
            Error::CrossbeamSend(err) => write!(f, "Error::CrossbeamSend({err})"),
            Error::TokioSend(err) => write!(f, "Error::TokioSend({err})"),
        }        
    }
}

impl Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} depth:{} packet:{:#?}", self.message, self.depth, self.packet)
    }
}

impl error::Error for Error {}
impl error::Error for PacketError {}

impl From<PacketError> for Error {
    fn from(value: PacketError) -> Self {
        Self::Packet(value)
    }
}

impl From<crossbeam::channel::SendError<Packet>> for Error {
    fn from(value: crossbeam::channel::SendError<Packet>) -> Self {
        Self::CrossbeamSend(value)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Packet>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<Packet>) -> Self {
        Self::TokioSend(value)
    }
}