use std::{any::type_name_of_val, future::Future, io::Cursor, ops::{Add, Sub}, pin::Pin, task::{Context, Poll}, time::Instant};
use crate::prelude::*;

const ALT_DEPTH_SIZE: usize = 4;

pub type AltIndex = u32;
pub type AltSize = u32;

pub type IdDelta = u32;
pub type IdSize = u32;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(Consist, Transient);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Consist(Id, AltDepth);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u32);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AltDepth(u32);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Transient([AltIndex; ALT_DEPTH_SIZE]);

#[derive(Default, Debug, Clone)]
pub struct Payload(Option<Box<[u8]>>);

#[derive(Debug, Clone)]
pub struct Packet {
    key: Key,
    payload: Payload,
}

#[derive(Debug)]
pub enum MessagePacket<S: State> {
    Message(MessagePacketMessage<S>),
    Carry(MessagePacketCarry<S>),
    Future(MessagePacketFuture<S>),
}

#[derive(Debug, Clone)]
pub struct MessagePacketMessage<S: State>{
    pub key: Key,
    pub instant: Option<Instant>,
    pub message: S::Message,
}

pub struct MessagePacketCarry<S: State>{
    pub key: Key,
    pub instant: Instant,
    pub lookup: Box<dyn Fn() -> S::Message + 'static + Send + Sync>,
}

pub struct MessagePacketFuture<S: State>{
    pub key: Key,
    pub instant: Instant,
    future: Pin<Box<dyn Future<Output = S::Message> + Send + Sync>>,
}

impl<S: State + std::fmt::Debug> std::fmt::Debug for MessagePacketCarry<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessagePacketCarry")
        .field("key", &self.key)
        .field("instant", &self.instant)
        .field("lookup", &type_name_of_val(&self.lookup))
        .finish()
    }
}

impl<S: State + std::fmt::Debug> std::fmt::Debug for MessagePacketFuture<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessagePacketFuture")
        .field("key", &self.key)
        .field("instant", &self.instant)
        .field("future", &type_name_of_val(&self.future))
        .finish()
    }
}

impl<S: State> Future for MessagePacketFuture<S> {
    type Output = MessagePacketMessage<S>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let key = self.key;
        let instant = Some(self.instant);

        self.get_mut().future.as_mut().poll(cx)
        .map(|message| MessagePacketMessage {
            key,
            instant, 
            message,
        })
    }
}

impl<S: State> MessagePacket<S> {
    pub fn message(key: Key, message: S::Message) -> Self {
        Self::Message(MessagePacketMessage { 
            key, 
            instant: None, 
            message, 
        })
    }

    pub fn carry<F>(key: Key, lookup: F) -> Self 
    where F: Fn() -> S::Message + 'static + Send + Sync {
        Self::Carry(MessagePacketCarry { 
            key, 
            instant: Instant::now(), 
            lookup: Box::new(lookup), 
        })
    }

    pub fn future<F>(key: Key, future: F) -> Self 
    where F: Future<Output = S::Message> + 'static + Send + Sync {
        Self::Future(MessagePacketFuture { 
            key, 
            instant: Instant::now(), 
            future: Box::pin(future), 
        })
    }

    pub fn wrap<P: State>(
        self,
        alt_depth: AltDepth,
        wrap: fn(AltIndex, S::Message) -> P::Message,
    ) -> MessagePacket<P> {        
        match self {
            Self::Message(message) => {
                let index = message.key.transient().index(alt_depth);

                MessagePacket::Message(MessagePacketMessage { 
                    key: message.key, 
                    instant: message.instant, 
                    message: wrap(index, message.message),
                })
            },
            Self::Carry(message) => {
                let index = message.key.transient().index(alt_depth);

                MessagePacket::Carry(MessagePacketCarry { 
                    key: message.key, 
                    instant: message.instant, 
                    lookup: Box::new(move || wrap(index, (message.lookup)())),
                })
            },
            Self::Future(message) => {
                let index = message.key.transient().index(alt_depth);

                MessagePacket::Future(MessagePacketFuture { 
                    key: message.key, 
                    instant: message.instant, 
                    future: Box::pin(async move { 
                        wrap(index, message.future.await) 
                    })
                })
            },
        }
    }
}

impl Key {
    pub fn consist(&self) -> Consist { self.0 }
    pub fn transient(&self) -> Transient { self.1 }

    pub fn new(
        consist: Consist, 
        transient: Transient,
    ) -> Self {
        Self(consist, transient)
    }

    pub fn access(
        mut self, 
        id_delta: IdDelta,
        alt_size: AltSize,
    ) -> Self {
        self.0 = self.0.access(id_delta, alt_size);
        self
    }

    pub fn alt(
        mut self, 
        index: AltIndex,
    ) -> Self {
        self.1 = self.transient().alt(self.consist().alt_depth(), index);
        self
    }
}

impl Consist {
    pub fn id(&self) -> Id { self.0 }
    pub fn alt_depth(&self) -> AltDepth { self.1 }

    pub fn new(
        id: Id, 
        alt_depth: AltDepth,
    ) -> Self {
        Self(id, alt_depth)
    }

    pub fn access(
        mut self, 
        id_delta: IdDelta,
        alt_size: AltSize,
    ) -> Self {
        self.0 = self.0 + id_delta;
        self.1 = self.1 + alt_size;
        self
    }
}

impl Transient {
    pub fn index(&self, alt_depth: AltDepth) -> AltIndex { 
        self.0[alt_depth.0 as usize]
    }

    pub fn alt(
        mut self, 
        depth: AltDepth,
        index: AltIndex,
    ) -> Self {
        self.0[depth.0 as usize] = index;
        self
    }
}

impl Packet {
    pub fn key(&self) -> Key { self.key }
    pub fn payload(&self) -> &Payload { &self.payload }

    pub fn new(key: Key, payload: Payload) -> Self {
        Self { 
            key, 
            payload, 
        }
    }
}

impl Payload {
    pub fn from_state<S: State>(state: &S) -> Self {
        let mut buffer = Vec::new();

        ciborium::into_writer(state, &mut buffer)
        .unwrap_or_else(|err| 
            panic!("serialize {:#?} into CBOR -> Err({err})", state)
        );

        Self(Some(buffer.into_boxed_slice()))
    }

    pub fn to_state<S: State>(&self) -> S {
        ciborium::from_reader(Cursor::new(self.0.as_ref().unwrap()))
        .unwrap_or_else(|err| 
            panic!("deserialize CBOR with {:#?} -> Err({err})", self)
        )
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

impl Add<u32> for AltDepth {
    type Output = Self;
    fn add(mut self, rhs: u32) -> Self::Output {
        self.0 = self.0.checked_add(rhs).unwrap();
        self
    }
}
