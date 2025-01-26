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
    id_delta: Option<IdDelta>,
    message: String,
}

#[derive(Debug, Clone)]
pub struct MessageError {
    key: Key,
    id_delta: Option<IdDelta>,
    message: String,
}

impl PacketError {
    pub fn new(
        packet: Packet,
        id_delta: Option<IdDelta>,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            packet,
            id_delta,
            message: message.as_ref().to_string(),
        }
    }
}

impl MessageError {
    pub fn new(
        key: Key,
        id_delta: Option<IdDelta>,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            key,
            id_delta,
            message: message.as_ref().to_string(),
        }
    }

    pub fn log_error(result: Result<(), MessageError>) {
        if let Err(err) = result {
            log::error!("{err}")
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
        write!(f, "{} id_delta:{:?} packet:{:?}", self.message, self.id_delta, self.packet)
    }
}

impl Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} id_delta:{:?} key:{:?}", self.message, self.id_delta, self.key)
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

impl From<crossbeam::channel::SendError<PacketMessage>> for MessageError {
    fn from(packet: crossbeam::channel::SendError<PacketMessage>) -> Self {
        MessageError {
            key: packet.0.key(),
            id_delta: None,
            message: format!("{}", packet),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<PacketMessage>> for MessageError {
    fn from(packet: tokio::sync::mpsc::error::SendError<PacketMessage>) -> Self {
        MessageError {
            key: packet.0.key(),
            id_delta: None,
            message: format!("{}", packet),
        }
    }
}

impl From<crossbeam::channel::SendError<FutureMessage>> for MessageError {
    fn from(packet: crossbeam::channel::SendError<FutureMessage>) -> Self {
        MessageError {
            key: packet.0.0,
            id_delta: None,
            message: format!("future"),
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<FutureMessage>> for MessageError {
    fn from(packet: tokio::sync::mpsc::error::SendError<FutureMessage>) -> Self {
        MessageError {
            key: packet.0.0,
            id_delta: None,
            message: format!("future"),
        }
    }
}

impl From<crossbeam::channel::SendError<PacketMessage>> for Error {
    fn from(packet: crossbeam::channel::SendError<PacketMessage>) -> Self {
        Error::Message(packet.into())
    }
}

impl From<tokio::sync::mpsc::error::SendError<PacketMessage>> for Error {
    fn from(packet: tokio::sync::mpsc::error::SendError<PacketMessage>) -> Self {
        Error::Message(packet.into())
    }
}

impl From<crossbeam::channel::SendError<FutureMessage>> for Error {
    fn from(packet: crossbeam::channel::SendError<FutureMessage>) -> Self {
        Error::Message(packet.into())
    }
}

impl From<tokio::sync::mpsc::error::SendError<FutureMessage>> for Error {
    fn from(packet: tokio::sync::mpsc::error::SendError<FutureMessage>) -> Self {
        Error::Message(packet.into())
    }
}
