use std::sync::{atomic::AtomicBool, Arc};
use bases::*;
use extends::AtomicNode;
use crate::*;

const IS_SOME_ID: NodeId = 0;
const ITEM_ID: NodeId = 1;

#[derive(Debug, Clone)]
pub enum OptionMessage<A: Accessor> {
    Item(Option<A::Message>),
    IsSome(bool),
    State(Option<A::State>),
}

#[derive(Debug, Clone)]
pub struct OptionNode<A: Accessor> { 
    key: NodeKey,
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
        packet: &PacketMessage, 
        depth: usize, 
    ) -> Result<Self, MessageError> {                    
        match packet.get_id(depth) {
            Some(IS_SOME_ID) => Ok(Self::IsSome(
                <bool as Accessor>::Message::from_packet_message(packet, depth + 1)?
            )),
            Some(ITEM_ID) => Ok(Self::Item(
                S::Message::from_packet_message(packet, depth + 1).ok()
            )),
            Some(_) => Err(MessageError::new(
                packet.key().clone(),
                depth,
                "unknown id",
            )),
            None => Ok(Self::State(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
        }
    }

    fn from_packet(
        packet: &Packet, 
        depth: usize, 
    ) -> Result<Self, PacketError> {                    
        match packet.get_id(depth) {
            Some(IS_SOME_ID) => Ok(Self::IsSome(<bool as Accessor>::Message::from_packet(packet, depth + 1)?)),
            Some(ITEM_ID) => Ok(Self::Item(S::Message::from_packet(packet, depth + 1).ok())),
            Some(_) => Err(PacketError::new(
                packet.clone(),
                depth,
                "unknown id",
            )),
            None => Ok(Self::State(
                packet.read_state()
            )),
        }
    }

    fn to_packet(
        &self,
        header: &Header, 
    ) -> Result<Packet, MessageError> {
        match self {
            Self::Item(Some(message)) => message.to_packet(header),
            Self::Item(None) => Ok(Packet::new(header.clone(), &())),
            Self::IsSome(is_some) => Ok(Packet::new(header.clone(), is_some)),
            Self::State(state) => Ok(Packet::new(header.clone(), state)),
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
    fn default() -> Self { Self::new(vec![], None, None) }
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
    fn key(&self) -> &NodeKey { &self.key }
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
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
        emitter: Option<&Emitter>,
    ) -> Self {        
        if let Some(id) = id { key.push(id); }
        
        Self { 
            key: key.clone().into_boxed_slice(),   
            emitter: emitter.cloned(),
            is_some: NewNode::new(key.clone(), Some(IS_SOME_ID), emitter),
            item: NewNode::new(key.clone(), Some(ITEM_ID), emitter),
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
        emitter: Option<&Emitter>,
    ) -> Self {
        Self {
            key: node.key.clone(),
            emitter: emitter.cloned(),
            is_some: Consensus::new_from(&node.is_some, emitter),
            item: Consensus::new_from(&node.item, emitter),
        }
    }

    fn set_emitter(&mut self, emitter: Option<&Emitter>) { 
        self.emitter = emitter.cloned(); 
        self.is_some.set_emitter(emitter);
        self.item.set_emitter(emitter);
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
            OptionMessage::State(state) => self.apply_state(state),
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