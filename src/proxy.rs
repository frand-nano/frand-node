use crate::ext::*;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Proxy<S: State> {
    _phantom: PhantomData<S>,
}

pub mod proxy {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum Message<S: State> {
        State(Proxy<S>),
    }

    #[derive(Debug, Clone)]
    pub struct Emitter<S: State> {
        callback: super::Callback<Proxy<S>>,
        subject: Arc<OnceLock<(Transient, S::Emitter)>>,
    }

    #[derive(Debug, Clone)]
    pub struct Accesser<S: State> {
        lookup: super::Lookup<Proxy<S>>,
        subject: Arc<OnceLock<(Transient, S::Accesser)>>,
    }

    #[derive(Debug, Clone)]
    pub struct Node<'n, S: State> {
        accesser: &'n Accesser<S>,
        emitter: &'n Emitter<S>,
        callback_mode: &'n CallbackMode,
        transient: &'n super::Transient,
    }

    impl<S: State> super::State for Proxy<S> {
        const NODE_SIZE: super::IdSize = 1;
        const NODE_ALT_SIZE: super::AltSize = 0;

        type Message = proxy::Message<S>;
        type Emitter = proxy::Emitter<S>;
        type Accesser = proxy::Accesser<S>;
        type Node<'n> = proxy::Node<'n, S>;

        fn from_payload(payload: &super::Payload) -> Self {
            super::Payload::to_state(payload)
        }

        fn to_payload(&self) -> super::Payload {
            super::Payload::from_state(self)
        }

        fn into_message(self) -> Self::Message {
            Self::Message::State(self)
        }
    }

    impl<S: State> super::Fallback for Proxy<S> {
        fn fallback(_node: Node<'_, S>, message: Message<S>, _delta: Option<std::time::Duration>) {
            match message {
                Message::State(_state) => {}
            }
        }
    }

    impl<S: State> super::System for Proxy<S> {

    }

    impl<S: State> super::Message for Message<S> {
        type State = Proxy<S>;

        fn from_packet(
            packet: &super::Packet,
            parent_key: super::Key,
            depth: usize,
        ) -> super::Result<Self> {
            Ok(
                match packet.key().consist().id() - parent_key.consist().id() {
                    0 => Ok(Self::State(super::State::from_payload(packet.payload()))),
                    id_delta => Err(super::PacketError::new(
                        packet.clone(),
                        Some(id_delta),
                        Some(depth),
                        format!("{}: unknown id_delta", std::any::type_name::<Self>(),),
                    )),
                }?,
            )
        }

        fn to_packet(&self, key: super::Key) -> super::Packet {
            match self {
                Self::State(state) => super::Packet::new(key, super::State::to_payload(state)),
            }
        }

        fn apply_to(&self, state: &mut Proxy<S>) {
            match self {
                Self::State(new_state) => *state = new_state.clone(),
            }
        }
    }

    impl<S: State> super::Emitter<Proxy<S>> for Emitter<S> {
        fn callback(&self) -> &super::Callback<Proxy<S>> {
            &self.callback
        }

        fn new(callback: super::Callback<Proxy<S>>) -> Self {
            Self { 
                callback,
                subject: Arc::new(OnceLock::new()),
            }
        }
    }

    impl<S: State> super::Accesser<Proxy<S>> for Accesser<S> {
        fn lookup(&self) -> &super::Lookup<Proxy<S>> {
            &self.lookup
        }

        fn new<CS: super::System>(builder: super::LookupBuilder<CS, Proxy<S>>) -> Self {
            Self {
                lookup: builder.build(|state| state.cloned()),
                subject: Arc::new(OnceLock::new()),
            }
        }
    }

    impl<'n, S: State> super::Node<'n, Proxy<S>> for Node<'n, S> {
        fn accesser(&self) -> &Accesser<S> {
            self.accesser
        }

        fn emitter(&self) -> &Emitter<S> {
            self.emitter
        }

        fn callback_mode(&self) -> &CallbackMode {
            self.callback_mode
        }

        fn transient(&self) -> &super::Transient {
            self.transient
        }
    }

    impl<'n, S: State> super::NewNode<'n, Proxy<S>> for Node<'n, S> {
        fn new(
            accesser: &'n Accesser<S>,
            emitter: &'n Emitter<S>,
            callback_mode: &'n CallbackMode,
            transient: &'n super::Transient,
        ) -> Self {
            Self {
                accesser,
                emitter,
                callback_mode,
                transient,
            }
        }
    }

    impl<'n, S: State> Node<'n, S> {
        pub fn set_subject(&'n self, node: S::Node<'_>) -> Result<(), Key> {
            use super::Node;

            self.accesser.subject.set(
                (*node.transient(), node.accesser().clone())
            ).map_err(|_| node.key())?;

            self.emitter.subject.set(
                (*node.transient(), node.emitter().clone())
            ).map_err(|_| node.key())?;

            Ok(())
        }

        pub fn subject(&'n self) -> Option<S::Node<'n>> {
            if let (
                Some((transient, accesser)), 
                Some((_, emitter)),
            ) = (
                self.accesser.subject.get(),
                self.emitter.subject.get(),
            ) {
                Some(super::NewNode::new(
                    accesser, 
                    emitter, 
                    &CallbackMode::Default,
                    transient,
                ))
            } else {
                None
            }
        }
    }
}
