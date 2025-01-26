use std::{fmt::Debug, io::Cursor, ops::{Add, Sub}, time::Instant};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use super::*;

const COLLECTION_DEPTH_SIZE: usize = 4;

pub type Index = u32;
pub type IdDelta = u32;
pub type Payload = Box<[u8]>;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(u32);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Depth(usize);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key(Id, [Index; COLLECTION_DEPTH_SIZE]);

#[derive(Debug, Clone)]
pub struct Packet {
    key: Key,
    payload: Payload,
}

#[derive(Debug)]
pub struct PacketMessage {
    key: Key,
    payload: Box<dyn Emitable>,
    carry: Option<Instant>, 
}

impl Packet {
    pub fn key(&self) -> Key { self.key }

    pub fn new<S: State + Serialize>(
        key: Key, 
        state: &S,
    ) -> Self {
        let mut buffer = Vec::new();

        ciborium::into_writer(state, &mut buffer)
        .unwrap_or_else(|err| 
            panic!("serialize {:#?} into CBOR -> Err({err})", state)
        );

        Packet {
            key,
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
        id_delta: Option<IdDelta>,
        message: impl AsRef<str>,
    ) -> PacketError {
        PacketError::new(self.clone(), id_delta, message)
    }
}

impl PacketMessage {
    pub fn key(&self) -> Key { self.key }
    pub fn payload(&self) -> &Box<dyn Emitable> { &self.payload }
    pub fn carry(&self) -> &Option<Instant> { &self.carry }

    pub fn set_carry(&mut self) { 
        self.carry = Some(Instant::now()); 
    }

    pub fn new(
        key: Key, 
        payload: Box<dyn Emitable>,
    ) -> Self {
        Self {
            key,
            payload,
            carry: None,
        } 
    }

    pub unsafe fn to_packet<S: State>(&self) -> Packet {
        Packet::new::<S>(
            self.key, 
            &S::from_emitable(&self.payload)
        )
    }
}

impl Key {
    pub fn id(&self) -> Id { self.0 }

    pub fn index(&self, depth: Depth) -> Index { 
        self.1[depth.0]
    }

    pub fn set_index(&mut self, depth: Depth, index: Index) { 
        self.1[depth.0] = index;
    }
}

impl Add<IdDelta> for Key {
    type Output = Self;
    fn add(mut self, id_delta: IdDelta) -> Self::Output {
        self.0 = self.0 + id_delta;
        self
    }
}

impl Sub<Id> for Id {
    type Output = IdDelta;
    fn sub(self, rhs: Self) -> Self::Output {
        self.0.checked_sub(rhs.0).unwrap()
    }
}

impl Add<IdDelta> for Id {
    type Output = Self;
    fn add(mut self, id_delta: IdDelta) -> Self::Output {
        self.0 = self.0.checked_add(id_delta).unwrap();
        self
    }
}

impl Add<usize> for Depth {
    type Output = Self;
    fn add(mut self, rhs: usize) -> Self::Output {
        self.0 = self.0.checked_add(rhs).unwrap();
        self
    }
}