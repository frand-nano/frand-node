use std::{fmt::Debug, io::Cursor, ops::{Add, Sub}, time::Instant};
use serde::{de::DeserializeOwned, Serialize};
use super::*;

pub type Index = u32;
pub type Payload = Box<[u8]>;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(u64, u64);

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
        index: Option<Index>,
        message: impl AsRef<str>,
    ) -> PacketError {
        PacketError::new(self.clone(), index, message)
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

impl From<u128> for Key {
    fn from(value: u128) -> Self {
        Self((value >> 64) as u64, value as u64)
    }
}

impl From<Key> for u128 {
    fn from(value: Key) -> Self {
        (value.0 as u128) << 64 | (value.1 as u128)
    }
}

impl From<Key> for (u64, u64) {
    fn from(value: Key) -> Self {
        (value.0, value.1)
    }
}

impl Add<Index> for Key {
    type Output = Self;
    fn add(self, rhs: Index) -> Self::Output {
        let mut combined: u128 = self.into();
        combined = combined.checked_add(rhs as u128).unwrap();
        combined.into()
    }
}

impl Sub<Index> for Key {
    type Output = Self;
    fn sub(self, rhs: Index) -> Self::Output {
        let mut combined: u128 = self.into();
        combined = combined.checked_sub(rhs as u128).unwrap();
        combined.into()
    }
}

impl Sub for Key {
    type Output = Index;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut combined: u128 = self.into();
        combined = combined.checked_sub(rhs.into()).unwrap();
        combined.try_into().unwrap()
    }
}