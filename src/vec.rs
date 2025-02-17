use std::vec::IntoIter;
use crate::ext::*;

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
        callback: super::Callback<Vec<I>>,
        pub push: super::Callback<I>,
        pub pop: super::Callback<()>,
        pub len: super::Callback<AltIndex>,
        pub item: <I as super::State>::Emitter,
    }

    #[derive(Debug, Clone)]
    pub struct Accesser<I: System> {
        lookup: super::Lookup<Vec<I>>,
        lookup_len: super::Lookup<usize>,
        pub item: <I as super::State>::Accesser,
    }

    #[derive(Debug, Clone)]
    pub struct Node<'n, I: System> {  
        accesser: &'n Accesser<I>,
        emitter: &'n Emitter<I>,
        callback_mode: &'n CallbackMode,
        transient: &'n super::Transient,
        pub item: <I as super::State>::Node<'n>,
    }

    impl<I: System> super::State for Vec<I> {
        const NODE_SIZE: super::IdSize = 1 + <super::AltIndex as super::State>::NODE_SIZE + <I as super::State>::NODE_SIZE;
        const NODE_ALT_SIZE: super::AltSize = 1;
    
        type Message = vec::Message<I>;
        type Emitter = vec::Emitter<I>;
        type Accesser = vec::Accesser<I>;
        type Node<'n> = vec::Node<'n, I>;
    
        fn from_payload(payload: &super::Payload) -> Self {
            super::Payload::to_state(payload)
        }
    
        fn to_payload(&self) -> super::Payload {
            super::Payload::from_state(self)
        }
    
        fn into_message(self) -> Self::Message {
            vec::Message::State(self)
        }
    }

    impl<I: System> super::Fallback for Vec<I> {
        fn fallback(
            node: Node<'_, I>,
            message: Message<I>,
            delta: Option<std::time::Duration>,
        ) {
            match message {
                Message::Push(_) => (),
                Message::Pop => (),
                Message::Len(_) => (),
                Message::Item(index, message) => {
                    I::handle(
                        node.item(index).node(),
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
                Self::Item(index, message) => message.to_packet(key.alt(*index)),
                Self::State(state) => super::Packet::new(key, super::State::to_payload(state)),
            }
        }

        fn apply_to(&self, state: &mut Vec<I>) {
            match self {
                Self::Push(item) => state.push(item.clone()),
                Self::Pop => { state.pop(); },
                Self::Len(len) => state.resize(*len as usize, Default::default()),
                Self::Item(index, message) => {
                    if let Some(item) = state.get_mut(*index as usize) {
                        message.apply_to(item);
                    }
                },
                Self::State(new_state) => *state = new_state.clone(),
            }
        }
    }

    impl<I: System> super::Emitter<Vec<I>> for Emitter<I> {
        fn callback(&self) -> &super::Callback<Vec<I>> {
            &self.callback
        }

        fn new(callback: super::Callback<Vec<I>>) -> Self {
            Self {
                push: super::Callback::<I>::access(
                    *callback.consist(),
                    callback.callback().clone(),
                    callback.process().clone(),
                    PUSH_ID_DELTA,
                    |_, message| {
                        let mut item = I::default();
                        super::Message::apply_to(&message, &mut item);
                        Message::Push(item)
                    },
                ),
                pop: super::Callback::access(
                    *callback.consist(),
                    callback.callback().clone(),
                    callback.process().clone(),
                    POP_ID_DELTA,
                    |_, _| Message::Pop,
                ),
                len: super::Callback::access(
                    *callback.consist(),
                    callback.callback().clone(),
                    callback.process().clone(),
                    LEN_ID_DELTA,
                    |_, message| Message::Len(message),
                ),
                item: super::Emitter::new(super::Callback::access(
                    *callback.consist(),
                    callback.callback().clone(),
                    callback.process().clone(),
                    ITEM_ID_DELTA,
                    |index, message| Message::Item(index, message),
                )),
                callback,
            }
        }
    }

    impl<I: System> super::Accesser<Vec<I>> for Accesser<I> {
        fn lookup(&self) -> &Lookup<Vec<I>> {
            &self.lookup
        }

        fn new<CS: System>(builder: LookupBuilder<CS, Vec<I>>) -> Self {
            Self { 
                item: super::Accesser::new(builder.access(
                    |state, index| state.map(|state| state.get(index as usize)).flatten(), 
                    ITEM_ID_DELTA
                )),
                lookup: builder.clone().build(|state| state.cloned()), 
                lookup_len: builder.build(|state| state.map(|state| state.len())), 
            }
        }
    }

    impl<'n, I: System> super::Node<'n, Vec<I>> for Node<'n, I> {
        fn accesser(&self) -> &Accesser<I> { self.accesser }
        fn emitter(&self) -> &Emitter<I> { self.emitter }
        fn callback_mode(&self) -> &CallbackMode { &self.callback_mode }
        fn transient(&self) -> &super::Transient { &self.transient }
    }
    
    impl<'n, I: System> super::NewNode<'n, Vec<I>> for Node<'n, I> {
        fn new(
            accesser: &'n Accesser<I>,
            emitter: &'n Emitter<I>,
            callback_mode: &'n CallbackMode,
            transient: &'n super::Transient,
        ) -> Self {
            Self {
                accesser,
                emitter,
                callback_mode,
                transient,
                item: super::NewNode::new(
                    &accesser.item, 
                    &emitter.item, 
                    callback_mode,
                    transient,
                ),
            }
        }
    }

    impl<'n, I: System> Node<'n, I> {
        pub fn emit_push(&self, item: I) {
            self.emitter.push.emit(self.callback_mode, self.transient, item.into_message());
        }

        pub fn emit_pop(&self) {
            self.emitter.pop.emit(self.callback_mode, self.transient, ());            
        }

        pub fn items(&self) -> IntoIter<NodeAlt<'_, I>> {
            let mut result = Vec::new();

            for index in 0..self.len() {
                result.push(self.item(index as u32))
            }                   

            result.into_iter()   
        }     

        pub fn len(&self) -> usize {
            self.accesser.lookup_len.get(self.transient).unwrap_or_default()
        }

        pub fn item(&self, index: AltIndex) -> NodeAlt<'_, I> {
            use crate::ext::Node;
            self.item.alt(self.consist(), index)
        }
    }
}
