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

pub enum MessagePacket<M: 'static> {
    Message(Key, Option<Instant>, M),
    Carry(Key, Instant, M),
    Future(Key, Instant, Pin<Box<dyn Future<Output = M> + Send + Sync>>),
}

impl<M: 'static + std::fmt::Debug> std::fmt::Debug for MessagePacket<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(arg0, arg1, arg2) => {
                f.debug_tuple("Message").field(arg0).field(arg1).field(arg2).finish()
            },
            Self::Carry(arg0, arg1, arg2) => {
                f.debug_tuple("Carry").field(arg0).field(arg1).field(arg2).finish()
            },
            Self::Future(arg0, arg1, arg2) => {
                f.debug_tuple("Future").field(arg0).field(arg1).field(&type_name_of_val(arg2)).finish()
            },
        }
    }
}

impl<M: 'static + Clone + Unpin> Future for MessagePacket<M> {
    type Output = Self;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            MessagePacket::Message(key, instant, message) => Poll::Ready(
                Self::Message(*key, *instant, message.clone())
            ),
            MessagePacket::Carry(key, instant, message) => Poll::Ready(
                Self::Message(*key, Some(*instant), message.clone())
            ),
            MessagePacket::Future(key, instant, future) => {
                future.as_mut().poll(cx)
                .map(|message| Self::Message(*key, Some(*instant), message.clone()))
            },
        }
    }
}

impl<M: 'static> MessagePacket<M> {
    pub fn new_future<F>(key: Key, future: F) -> Self 
    where F: Future<Output = M> + 'static + Send + Sync {
        Self::Future(key, Instant::now(), Box::pin(future))
    }

    pub fn wrap<P: 'static>(
        self,
        alt_depth: AltDepth,
        wrap: fn(AltIndex, M) -> P,
    ) -> MessagePacket<P> {        
        match self {
            Self::Message(key, instant, message) => {
                MessagePacket::Message(
                    key, 
                    instant, 
                    wrap(key.transient().index(alt_depth), message),
                )
            },
            Self::Carry(key, instant, message) => {
                MessagePacket::Carry(
                    key, 
                    instant, 
                    wrap(key.transient().index(alt_depth), message),
                )
            },
            Self::Future(key, instant, future) => {
                let index = key.transient().index(alt_depth);
                MessagePacket::Future(
                    key, 
                    instant, 
                    Box::pin(async move { 
                        wrap(index, future.await) 
                    })
                )
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
