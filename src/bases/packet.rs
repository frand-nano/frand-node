use std::{fmt::Debug, io::Cursor};
use serde::{de::DeserializeOwned, Serialize};

use super::*;

pub type NodeId = u32;
pub type NodeKey = Box<[NodeId]>;

pub type Header = NodeKey;
pub type Payload = Box<[u8]>;

#[derive(Debug, Clone)]
pub struct Packet {
    header: Header,
    payload: Payload,
}

#[derive(Debug, Clone)]
pub struct PacketMessage<M: Message> {
    pub header: Header,
    pub message: M,
}

impl Packet {
    pub fn key(&self) -> &NodeKey { &self.header }

    pub fn get_id(&self, depth: usize) -> Option<NodeId> { 
        self.key().get(depth).copied()
    }

    pub fn new<S: State + Serialize>(
        node_key: NodeKey, 
        state: &S,
    ) -> Self {
        let mut buffer = Vec::new();

        ciborium::into_writer(state, &mut buffer)
        .unwrap_or_else(|err| 
            panic!("serialize {:#?} into CBOR -> Err({err})", state)
        );

        Packet {
            header: node_key,
            payload: buffer.into_boxed_slice(),
        }      
    }

    pub fn read_state<S: State + DeserializeOwned>(&self) -> S {
        ciborium::from_reader(Cursor::new(&self.payload))
        .unwrap_or_else(|err| 
            panic!("deserialize CBOR with {:#?} -> Err({err})", self.payload)
        )
    }
    
    pub fn error(
        &self, 
        depth: usize,
        message: impl AsRef<str>,
    ) -> PacketError {
        PacketError::new(self.clone(), depth, message)
    }
}

impl<M: Message> PacketMessage<M> {
    pub fn to_packet<S: State>(&self) -> Result<Packet, MessageError> {
        self.message.to_packet(&self.header)
    }
}

impl<M: Message> TryFrom<&Packet> for PacketMessage<M> {
    type Error = PacketError;
    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        Ok(Self {
            header: packet.header.clone(),
            message: Message::from_packet(packet, 0)?,
        })
    }
}