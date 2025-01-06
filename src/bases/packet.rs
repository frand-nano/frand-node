use std::{fmt::Debug, io::Cursor, time::Instant};
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

#[derive(Debug)]
pub struct PacketMessage {
    header: Header,
    payload: Box<dyn Emitable>,
    carry: Option<Instant>, 
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

impl PacketMessage {
    pub fn key(&self) -> &NodeKey { &self.header }
    pub fn payload(&self) -> &Box<dyn Emitable> { &self.payload }
    pub fn carry(&self) -> &Option<Instant> { &self.carry }

    pub fn get_id(&self, depth: usize) -> Option<NodeId> { 
        self.header.get(depth).copied()
    }

    pub fn set_carry(&mut self) { 
        self.carry = Some(Instant::now()); 
    }

    pub fn new(
        node_key: NodeKey, 
        payload: Box<dyn Emitable>,
    ) -> Self {
        Self {
            header: node_key,
            payload,
            carry: None,
        } 
    }

    pub unsafe fn to_packet<S: State>(&self) -> Packet {
        Packet::new::<S>(
            self.header.clone(), 
            &S::from_emitable(&self.payload)
        )
    }
}