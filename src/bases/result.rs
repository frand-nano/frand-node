use core::error;
use std::fmt::{Debug, Display};
use super::*;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum Error {
    Text(String),
    Packet(PacketError),
    Message(MessageError),
}

#[derive(Debug, Clone)]
pub struct PacketError {
    packet: Packet,
    depth: usize,
    message: String,
}

#[derive(Debug, Clone)]
pub struct MessageError {
    header: Header,
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

impl MessageError {
    pub fn new(
        header: Header,
        depth: usize,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            header,
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
            Error::Message(err) => write!(f, "Error::Message({err})"),
        }        
    }
}

impl Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} depth:{} packet:{:#?}", self.message, self.depth, self.packet)
    }
}

impl Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} depth:{} header:{:#?}", self.message, self.depth, self.header)
    }
}

impl error::Error for Error {}
impl error::Error for PacketError {}
impl error::Error for MessageError {}

impl From<PacketError> for Error {
    fn from(value: PacketError) -> Self {
        Self::Packet(value)
    }
}

impl From<MessageError> for Error {
    fn from(value: MessageError) -> Self {
        Self::Message(value)
    }
}

impl<M: Message> From<crossbeam::channel::SendError<PacketMessage<M>>> for Error {
    fn from(packet: crossbeam::channel::SendError<PacketMessage<M>>) -> Self {
        Self::Message(MessageError {
            header: packet.0.header,
            depth: 0,
            message: format!("{:?}", packet.0.message),
        })
    }
}

impl<M: Message> From<tokio::sync::mpsc::error::SendError<PacketMessage<M>>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<PacketMessage<M>>) -> Self {
        Self::Text(value.to_string())
    }
}

impl<M: Message> From<crossbeam::channel::SendError<M>> for Error {
    fn from(value: crossbeam::channel::SendError<M>) -> Self {
        Self::Text(value.to_string())
    }
}

impl<M: Message> From<tokio::sync::mpsc::error::SendError<M>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<M>) -> Self {
        Self::Text(value.to_string())
    }
}