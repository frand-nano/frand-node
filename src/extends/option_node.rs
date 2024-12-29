use std::future::Future;
use bases::*;
use crate::*;

const IS_SOME_ID: NodeId = 0;
const ITEM_ID: NodeId = 1;

#[derive(Debug, Clone)]
pub enum OptionMessage<S: State> {
    #[allow(non_camel_case_types)] item(Option<S::Message>),
    IsSome(bool),
    State(Option<S>),
}

#[derive(Debug, Clone)]
pub struct OptionConsensus<M: Message, S: State> { 
    key: NodeKey,
    pub is_some: <bool as State>::Consensus<M>, 
    item: S::Consensus<M>,
}

#[derive(Debug, Clone)]
pub struct OptionNode<M: Message, S: State> { 
    key: NodeKey,
    callback: Callback<M>,
    future_callback: FutureCallback<M>,
    pub is_some: <bool as State>::Node<M>, 
    item: S::Node<M>,
}

impl<S: State> State for Option<S> {
    type Message = OptionMessage<S>;
    type Consensus<M: Message> = OptionConsensus<M, S>;
    type Node<M: Message> = OptionNode<M, S>; 

    fn apply(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::item(message) => if let (Some(item), Some(message)) = (self, message) {
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
    fn from_state<S2: State>(
        header: &Header, 
        depth: usize, 
        state: S2,
    ) -> Result<Self, MessageError> {                    
        match header.get(depth).copied() {
            Some(IS_SOME_ID) => Ok(Self::IsSome(<bool as State>::Message::from_state(header, depth + 1, state)?)),
            Some(ITEM_ID) => Ok(Self::item(S::Message::from_state(header, depth + 1, state).ok())),
            Some(_) => Err(MessageError::new(
                header.clone(),
                depth,
                "unknown id",
            )),
            None => Ok(Self::State(
                unsafe { Self::cast_state(state) }
            )),
        }
    }

    fn from_packet(
        packet: &Packet, 
        depth: usize, 
    ) -> Result<Self, PacketError> {                    
        match packet.get_id(depth) {
            Some(IS_SOME_ID) => Ok(Self::IsSome(<bool as State>::Message::from_packet(packet, depth + 1)?)),
            Some(ITEM_ID) => Ok(Self::item(S::Message::from_packet(packet, depth + 1).ok())),
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
            Self::item(Some(message)) => message.to_packet(header),
            Self::item(None) => Ok(Packet::new(header.clone(), &())),
            Self::IsSome(is_some) => Ok(Packet::new(header.clone(), is_some)),
            Self::State(state) => Ok(Packet::new(header.clone(), state)),
        }
    }
}

impl<M: Message, S: State> OptionConsensus<M, S> {
    pub fn item(&self) -> Option<&S::Consensus<M>> {
        match self.is_some.v() {
            true => Some(&self.item),
            false => None,
        }
    }
}

impl<M: Message, S: State> Default for OptionConsensus<M, S> 
where Option<S>: State<Message = OptionMessage<S>, Consensus<M> = Self, Node<M> = OptionNode<M, S>> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<M: Message, S: State> Consensus<M, Option<S>> for OptionConsensus<M, S> 
where Option<S>: State<Message = OptionMessage<S>, Consensus<M> = Self, Node<M> = OptionNode<M, S>> {    
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self {        
        if let Some(id) = id { key.push(id); }
        
        Self { 
            key: key.clone().into_boxed_slice(),   
            is_some: Consensus::new(key.clone(), Some(IS_SOME_ID)),
            item: Consensus::new(key.clone(), Some(ITEM_ID)),
        }
    }
    
    fn new_node(
        &self, 
        callback: &Callback<M>, 
        future_callback: &FutureCallback<M>,
    ) -> OptionNode<M, S> {
        Node::new_from(self, callback, future_callback)
    }
        
    fn clone_state(&self) -> Option<S> { 
        match self.is_some.v() {
            true => Some(self.item.clone_state()),
            false => None,
        }
    }

    fn apply(&mut self, message: OptionMessage<S>) {
        match message {
            OptionMessage::item(message) => if let Some(message) = message { 
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

    fn apply_state(&mut self, state: Option<S>) {
        match state {
            Some(state) => {
                self.is_some.apply_state(true);
                self.item.apply_state(state);
            },
            None => self.is_some.apply_state(false),
        }
    }
}

impl<M: Message, S: State> OptionNode<M, S> {
    pub fn item(&self) -> Option<&S::Node<M>> {
        match self.is_some.v() {
            true => Some(&self.item),
            false => None,
        }
    }
}

impl<M: Message, S: State> Node<M, Option<S>> for OptionNode<M, S> 
where Option<S>: State<Message = OptionMessage<S>, Consensus<M> = OptionConsensus<M, S>> {    
    type State = Option<S>;
    
    fn key(&self) -> &NodeKey { &self.key }

    fn new_from(
        consensus: &OptionConsensus<M, S>,
        callback: &Callback<M>,
        future_callback: &FutureCallback<M>,
    ) -> Self {
        Self { 
            key: consensus.key.clone(), 
            callback: callback.clone(), 
            future_callback: future_callback.clone(), 
            is_some: Node::new_from(&consensus.is_some, callback, future_callback),
            item: Node::new_from(&consensus.item, callback, future_callback),
        }
    }    

    fn clone_state(&self) -> Option<S> { 
        match self.is_some.v() {
            true => Some(self.item.clone_state()),
            false => None,
        }
    }
}

impl<M: Message, S: State> Emitter<M, Option<S>> for OptionNode<M, S> 
where Option<S>: State {      
    fn emit(&self, state: Option<S>) {
        self.callback.emit(self.key.clone(), state)
    }    

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = Option<S>> + Send {
        self.future_callback.emit(self.key.clone(), future)
    }
}