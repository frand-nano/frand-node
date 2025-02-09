use std::sync::{Arc, RwLockReadGuard};
use crate::ext::*;

impl<I: System> State for Vec<I> {
    const NODE_SIZE: IdSize = 1 + <AltIndex as State>::NODE_SIZE + <I as State>::NODE_SIZE;
    const NODE_ALT_SIZE: AltSize = 1;

    type Message = vec::Message<I>;
    type Emitter = vec::Emitter<I>;
    type Accesser<CS: System> = vec::Accesser<CS, I>;
    type Node<'n, CS: System> = vec::Node<'n, CS, I>;

    fn from_payload(payload: &Payload) -> Self {
        Payload::to_state(payload)
    }

    fn to_payload(&self) -> Payload {
        Payload::from_state(self)
    }

    fn into_message(self) -> Self::Message {
        vec::Message::State(self)
    }
}

pub mod vec {
    use super::*;

    const PUSH_ID_DELTA: super::IdDelta = 1;
    const PUSH_ID_DELTA_END: super::IdDelta = PUSH_ID_DELTA + 1;
    const POP_ID_DELTA: super::IdDelta = PUSH_ID_DELTA_END;
    const POP_ID_DELTA_END: super::IdDelta = POP_ID_DELTA + 1;
    const LEN_ID_DELTA: super::IdDelta = POP_ID_DELTA_END;
    const LEN_ID_DELTA_END: super::IdDelta = LEN_ID_DELTA + 1;
    const ITEM_ID_DELTA: super::IdDelta = LEN_ID_DELTA_END;

    #[derive(Debug, Clone)]
    pub enum Message<I: System> {
        Push(I),
        Pop,
        Len(<AltIndex as super::State>::Message),
        Item(AltIndex, <I as super::State>::Message),
        State(Vec<I>),
    }

    #[derive(Debug, Clone)]
    pub struct Emitter<I: System> {
        callback: super::Callback<Message<I>>,
        pub push: super::Callback<I>,
        pub pop: super::Callback<()>,
        pub len: super::Callback<AltIndex>,
        pub item: <I as super::State>::Emitter,
    }

    #[derive(Debug, Clone)]
    pub struct Accesser<CS: super::System, I: System> {
        access: super::RcAccess<Vec<I>, CS>,
        pub item: <I as super::State>::Accesser<CS>,
    }

    #[derive(Debug)]
    pub struct Node<'n, CS: super::System, I: System> {
        emitter: &'n Emitter<I>,
        accesser: &'n Accesser<CS, I>,
        consensus: &'n Arc<RwLockReadGuard<'n, CS>>,
        transient: &'n super::Transient,
        pub item: <I as super::State>::Node<'n, CS>,
    }

    impl<I: System> super::Fallback for Vec<I> {
        fn fallback<CS: super::System>(
            node: Node<'_, CS, I>,
            message: Message<I>,
            delta: Option<std::time::Duration>,
        ) {
            match message {
                Message::Push(_) => (),
                Message::Pop => (),
                Message::Len(_) => (),
                Message::Item(index, message) => {
                    let item = node.item(index);
                    <I>::handle(
                        item.node(),
                        message, 
                        delta,
                    )
                },
                Message::State(_) => (),
            }
        }
    }

    impl<I: System> super::System for Vec<I> {

    }

    impl<I: System> super::Message for Message<I> {
        type State = Vec<I>;

        fn from_packet(
            packet: &super::Packet,
            parent_key: super::Key,
            depth: usize,
        ) -> super::Result<Self> {
            Ok(
                match packet.key().consist().id() - parent_key.consist().id() {
                    0 => Ok(Self::State(
                        super::State::from_payload(packet.payload())
                    )),
                    PUSH_ID_DELTA..PUSH_ID_DELTA_END => Ok(Message::Push(
                        super::State::from_payload(packet.payload())
                    )),
                    POP_ID_DELTA..POP_ID_DELTA_END => Ok(Message::Pop),
                    LEN_ID_DELTA..LEN_ID_DELTA_END => Ok(Message::Len(
                        <AltIndex as super::State>::Message::from_packet(
                            packet,
                            super::Key::new(
                                parent_key
                                    .consist()
                                    .access(LEN_ID_DELTA, <Vec<I>>::NODE_ALT_SIZE),
                                parent_key.transient(),
                            ),
                            depth + 1,
                        )?,
                    )),
                    ITEM_ID_DELTA.. => {
                        Ok(Message::Item(
                            packet.key().transient().index(parent_key.consist().alt_depth()),
                            <I as super::State>::Message::from_packet(
                                packet,
                                super::Key::new(
                                    parent_key
                                        .consist()
                                        .access(ITEM_ID_DELTA, <Vec<I>>::NODE_ALT_SIZE),
                                    parent_key.transient(),
                                ),
                                depth + 1,
                            )?
                        ))
                    }
                }?,
            )
        }

        fn to_packet(&self, key: super::Key) -> super::Packet {
            match self {
                Self::Push(item) => super::Packet::new(key, super::State::to_payload(item)),
                Self::Pop => super::Packet::new(key, super::State::to_payload(&())),
                Self::Len(message) => message.to_packet(key),
                Self::Item(index, message) => message.to_packet(
                    Key::new(
                        key.consist(),
                        key.transient().alt(key.consist().alt_depth(), *index),
                    )
                ),
                Self::State(state) => super::Packet::new(key, super::State::to_payload(state)),
            }
        }

        fn apply_to(&self, state: &mut Vec<I>) {
            match self {
                Self::Push(item) => state.push(item.clone()),
                Self::Pop => { state.pop(); },
                Self::Len(len) => state.resize(*len as usize, Default::default()),
                Self::Item(index, item) => item.apply_to(&mut state[*index as usize]),
                Self::State(new_state) => *state = new_state.clone(),
            }
        }
    }

    impl<I: System> super::Emitter<Vec<I>> for Emitter<I> {
        fn callback(&self) -> &super::Callback<Message<I>> {
            &self.callback
        }

        fn new(callback: super::Callback<Message<I>>) -> Self {
            Self {
                push: super::Callback::<I>::access(
                    callback.clone(),
                    PUSH_ID_DELTA,
                    <Vec<I>>::NODE_ALT_SIZE,
                    |_, item| Message::Push(item),
                ),
                pop: super::Callback::access(
                    callback.clone(),
                    POP_ID_DELTA,
                    <Vec<I>>::NODE_ALT_SIZE,
                    |_, _| Message::Pop,
                ),
                len: super::Callback::access(
                    callback.clone(),
                    LEN_ID_DELTA,
                    <Vec<I>>::NODE_ALT_SIZE,
                    |_, message| Message::Len(message),
                ),
                item: super::Emitter::new(super::Callback::access(
                    callback.clone(),
                    ITEM_ID_DELTA,
                    <Vec<I>>::NODE_ALT_SIZE,
                    |index, message| Message::Item(index, message),
                )),
                callback,
            }
        }
    }

    impl<CS: System, I: System> std::ops::Deref for Accesser<CS, I> {
        type Target = RcAccess<Vec<I>, CS>;
        fn deref(&self) -> &Self::Target { &self.access }
    }

    impl<CS: System, I: System> super::Accesser<Vec<I>, CS> for Accesser<CS, I> {
        fn new(access: super::RcAccess<Vec<I>, CS>) -> Self {
            Self {
                item: super::Accesser::new(super::RcAccess::access(
                    access.clone(),
                    ITEM_ID_DELTA,
                    <Vec<I>>::NODE_ALT_SIZE,
                    |state, index| &state[index as usize],
                )),
                access,
            }
        }
    }

    impl<'n, CS: super::System, I: System> std::ops::Deref for Node<'n, CS, I> {
        type Target = Vec<I>;

        fn deref(&self) -> &Self::Target {
            (self.accesser.access)(self.consensus, *self.transient)
        }
    }

    impl<'n, CS: super::System, I: System> super::Node<'n, Vec<I>> for Node<'n, CS, I> {
        fn transient(&self) -> &super::Transient {
            self.transient
        }

        fn emitter(&self) -> &Emitter<I> {
            self.emitter
        }
    }
    
    impl<'n, CS: super::System, I: System> super::NewNode<'n, Vec<I>, CS> for Node<'n, CS, I> {
        fn new(
            emitter: &'n Emitter<I>,
            accesser: &'n Accesser<CS, I>,
            consensus: &'n Arc<RwLockReadGuard<'n, CS>>,
            transient: &'n super::Transient,
        ) -> Self {
            Self {
                emitter,
                accesser,
                item: super::NewNode::new(&emitter.item, &accesser.item, consensus, transient),
                consensus,
                transient,
            }
        }
        
        fn alt(
            &self,
            transient: Transient,             
        ) -> ConsensusRead<'n, Vec<I>, CS> {
            ConsensusRead::new(
                self.emitter, 
                self.accesser, 
                self.consensus.clone(), 
                transient,
            )
        }
    }

    impl<'n, CS: super::System, I: System> Node<'n, CS, I> {
        pub fn emit_push(&self, item: I) {
            self.emitter.push.emit(self.transient, item);
        }

        pub fn emit_pop(&self) {
            self.emitter.pop.emit(self.transient, ());            
        }

        pub fn items(&self) -> std::vec::IntoIter<ConsensusRead<'_, I, CS>> {
            let mut result = Vec::new();

            for index in 0..self.len() {
                result.push(self.item(index as u32))
            }

            result.into_iter()
        }

        pub fn item(&self, index: AltIndex) -> ConsensusRead<'_, I, CS> {
            use crate::ext::Node;

            self.item.alt(
                (*self.transient()).alt(
                    self.consist().alt_depth(), 
                    index,
                )
            )
        }
    }
}
