use std::sync::{atomic::AtomicBool, Arc};
use bases::*;
use extends::AtomicNode;
use crate::*;

const IS_SOME_DELTA: IdDelta = 1;
const ITEM_DELTA: IdDelta = 2;
const NODE_SIZE: IdDelta = 2;

#[derive(Debug, Clone)]
pub enum OptionMessage<A: Accessor> {
    Item(Option<A::Message>),
    IsSome(bool),
    State(Option<A::State>),
}

#[derive(Debug, Clone)]
pub struct OptionNode<A: Accessor> { 
    key: Key,
    emitter: Option<Emitter>,
    pub is_some: AtomicNode<bool, Arc<AtomicBool>>, 
    item: A::Node,
}

impl<A: Accessor> Accessor for Option<A> {
    type State = Option<A::State>;
    type Message = OptionMessage<A::State>;
    type Node = OptionNode<A::State>; 
}

impl<S: State> Emitable for Option<S> {}

impl<S: State> State for Option<S> {
    const NODE_SIZE: IdDelta = NODE_SIZE + S::NODE_SIZE; 

    fn apply(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::Item(message) => if let (Some(item), Some(message)) = (self, message) {
                item.apply(message)
            }, 
            Self::Message::IsSome(is_some) => {
                if is_some != self.is_some() {
                    match is_some {
                        true => *self = Some(S::default()),
                        false => *self = None,
                    }
                }
            },
            Self::Message::State(state) => *self = state,
        }
    }
}

impl<S: State> Message for OptionMessage<S> {
    fn from_packet_message(
        parent_key: Key,
        depth: Depth,
        packet: &PacketMessage, 
    ) -> Result<Self, MessageError> {     
        match packet.key().id() - parent_key.id() {
            0 => Ok(Self::State(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
            IS_SOME_DELTA => Ok(Self::IsSome(
                <bool as Accessor>::Message::from_packet_message(
                    parent_key + IS_SOME_DELTA,
                    depth,
                    packet, 
                )?
            )),
            _ => Ok(Self::Item(
                S::Message::from_packet_message(
                    parent_key + ITEM_DELTA,
                    depth,
                    packet,
                ).ok()
            )),
        }
    }

    fn from_packet(
        parent_key: Key,
        depth: Depth,
        packet: &Packet, 
    ) -> Result<Self, PacketError> {        
        match packet.key().id() - parent_key.id() {
            0 => Ok(Self::State(
                packet.read_state()
            )),
            IS_SOME_DELTA => Ok(Self::IsSome(
                <bool as Accessor>::Message::from_packet(
                    parent_key + IS_SOME_DELTA,
                    depth,
                    packet, 
                )?
            )),
            _ => Ok(Self::Item(
                S::Message::from_packet(
                    parent_key + ITEM_DELTA,
                    depth,
                    packet,
                ).ok()
            )),
        }
    }

    fn to_packet(
        &self,
        key: Key, 
    ) -> Result<Packet, MessageError> {
        match self {
            Self::Item(Some(message)) => message.to_packet(key),
            Self::Item(None) => Ok(Packet::new(key, &())),
            Self::IsSome(is_some) => Ok(Packet::new(key, is_some)),
            Self::State(state) => Ok(Packet::new(key, state)),
        }
    }
}

impl<S: State> OptionNode<S> {
    pub fn item(&self) -> Option<&S::Node> {
        match self.is_some.v() {
            true => Some(&self.item),
            false => None,
        }
    }
}

impl<S: State> Default for OptionNode<S> 
where Option<S>: State<Message = OptionMessage<S>> {    
    fn default() -> Self { Self::new(
        Key::default(), 
        IdDelta::default(), 
        Depth::default(), 
        None,
    ) }
}

impl<S: State> Accessor for OptionNode<S>  {
    type State = Option<S>;
    type Message = OptionMessage<S>;     
    type Node = Self;
}

impl<S: State> Fallback for OptionNode<S> {
    fn fallback(&self, message: OptionMessage<S>, delta: Option<f32>) {
        use OptionMessage::*;
        match message {
            Item(Some(message)) => self.item.handle(message, delta),
            Item(None) => (),
            IsSome(message) => self.is_some.handle(message, delta),
            State(_) => (),
        }
    }
}

impl<S: State> System for OptionNode<S> {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        self.fallback(message, delta);
    }
}

impl<S: State> Node<Option<S>> for OptionNode<S> 
where Option<S>: State<Message = OptionMessage<S>> {  
    fn key(&self) -> Key { self.key }
    fn emitter(&self) -> Option<&Emitter> { self.emitter.as_ref() }

    fn clone_state(&self) -> Option<S> { 
        match self.is_some.v() {
            true => Some(self.item.clone_state()),
            false => None,
        }
    }
}

impl<S: State> NewNode<Option<S>> for OptionNode<S> 
where Option<S>: State<Message = OptionMessage<S>> {  
    fn new(
        mut key: Key,
        id_delta: IdDelta,
        depth: Depth,
        emitter: Option<Emitter>,
    ) -> Self {        
        key = key + id_delta;
        
        Self { 
            key,   
            emitter: emitter.clone(),
            is_some: NewNode::new(key, IS_SOME_DELTA, depth, emitter.clone()),
            item: NewNode::new(key, ITEM_DELTA, depth, emitter.clone()),
        }
    }
}

impl<S: State> Consensus<Option<S>> for OptionNode<S> 
where 
S::Node: Consensus<S>,
Option<S>: State<Message = OptionMessage<S>>, 
{  
    fn new_from(
        node: &Self,
        emitter: Option<Emitter>,
    ) -> Self {
        Self {
            key: node.key,
            emitter: emitter.clone(),
            is_some: Consensus::new_from(&node.is_some, emitter.clone()),
            item: Consensus::new_from(&node.item, emitter.clone()),
        }
    }

    fn set_emitter(&mut self, emitter: Option<Emitter>) { 
        self.emitter = emitter.clone(); 
        self.is_some.set_emitter(emitter.clone());
        self.item.set_emitter(emitter.clone());
    }

    fn apply(&self, message: OptionMessage<S>) {
        match message {
            OptionMessage::Item(message) => if let Some(message) = message { 
                if self.is_some.v() {
                    self.item.apply(message)
                }
            },
            OptionMessage::IsSome(is_some) => {
                if is_some != self.is_some.v() {
                    self.is_some.apply(is_some);
                    if is_some {
                        self.item.apply_state(S::default());
                    }
                }
            },
            OptionMessage::State(state) => {
                self.apply_state(state.clone());

                self.is_some.emit(state.is_some());
                if let Some(item) = state {
                    self.item.emit(item);
                }           
            },
        }    
    }

    fn apply_state(&self, state: Option<S>) {
        match state {
            Some(state) => {
                self.is_some.apply_state(true);
                self.item.apply_state(state);
            },
            None => self.is_some.apply_state(false),
        }
    }
}