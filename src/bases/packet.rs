use std::io::Cursor;
use anyhow::{anyhow, Error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use super::{Emitable, State};

pub type AnchorId = u32;
pub type AnchorKey = Box<[AnchorId]>;

pub type Header = AnchorKey;
pub type Payload = Box<[u8]>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Packet {
    header: Header,
    payload: Payload,
}

impl Packet {
    pub fn key(&self) -> &AnchorKey { &self.header }

    pub fn get_id(&self, depth: usize) -> Option<AnchorId> { 
        self.key().get(depth).copied()
    }

    pub fn new<E: Emitable + Serialize>(
        anchor_key: &AnchorKey, 
        emitable: E,
    ) -> Self {
        let mut buffer = Vec::new();

        ciborium::into_writer(&emitable, &mut buffer)
        .unwrap_or_else(|err| 
            panic!("serialize {:#?} into CBOR -> Err({err})", &emitable)
        );

        Packet {
            header: anchor_key.to_owned(),
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
    ) -> Error {
        anyhow!("{} depth:{depth} packet:{:#?}", message.as_ref(), self)
    }
}