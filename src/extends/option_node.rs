use std::future::Future;
use bases::{Node, NodeId, NodeKey, Packet, PacketError, Reporter};
use crate::*;

const IS_SOME_ID: NodeId = 0;
const ITEM_ID: NodeId = 1;

#[derive(Debug,Clone)]
pub enum OptionMessage<S: State> {
    #[allow(non_camel_case_types)] item(Option<S::Message>),
    IsSome(bool),
    State(Option<S>),
}

#[derive(Debug, Clone)]
pub struct OptionConsensus<S: State> { 
    key: NodeKey,
    pub is_some: <bool as State>::Consensus, 
    item: S::Consensus,
}

#[derive(Debug, Clone)]
pub struct OptionNode<S: State> { 
    key: NodeKey,
    reporter: Reporter,
    pub is_some: <bool as State>::Node, 
    item: S::Node,
}

impl<S: State> State for Option<S> {
    type Message = OptionMessage<S>;
    type Consensus = OptionConsensus<S>;
    type Node = OptionNode<S>; 

    fn apply(
        &mut self, 
        depth: usize, 
        packet: Packet,
    ) -> Result<(), PacketError>  {
        match packet.get_id(depth) {
            Some(ITEM_ID) => match self {
                Some(item) => item.apply(depth+1, packet),
                None => Ok(()),
            },
            Some(IS_SOME_ID) => {
                let is_some: bool = packet.read_state();
                Ok(if is_some != self.is_some() {
                    match is_some {
                        true => *self = Some(S::default()),
                        false => *self = None,
                    }
                })                                
            },
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => Ok(*self = packet.read_state()),
        }
    }    

    fn apply_message(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::item(message) => match (self, message) {
                (Some(item), Some(message)) => item.apply_message(message),
                _ => {},
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
    fn from_packet(
        depth: usize,
        packet: &Packet,
    ) -> Result<Self, PacketError> {
        match packet.get_id(depth) {
            Some(ITEM_ID) => Ok(Self::item(Some(S::Message::from_packet(depth + 1, packet)?))),
            Some(IS_SOME_ID) => Ok(Self::IsSome(bool::from_packet(depth + 1, packet)?)),
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => Ok(Self::State(packet.read_state())),
        }
    }
}

impl<S: State> OptionConsensus<S> {
    pub fn item(&self) -> Option<&S::Consensus> {
        match self.is_some.v() {
            true => Some(&self.item),
            false => None,
        }
    }
}

impl<S: State> Default for OptionConsensus<S> 
where Option<S>: State<Message = OptionMessage<S>, Consensus = Self, Node = OptionNode<S>> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<S: State> Consensus<Option<S>> for OptionConsensus<S> 
where Option<S>: State<Message = OptionMessage<S>, Consensus = Self, Node = OptionNode<S>> {    
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
    
    fn new_node(&self, reporter: &Reporter) -> OptionNode<S> {
        Node::new_from(self, reporter)
    }
        
    fn clone_state(&self) -> Option<S> { 
        match self.is_some.v() {
            true => Some(self.item.clone_state()),
            false => None,
        }
    }

    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<(), PacketError> {
        Ok(match packet.get_id(depth) {
            Some(ITEM_ID) => {
                if self.is_some.v() {
                    self.item.apply(depth+1, packet)?;
                }
            },
            Some(IS_SOME_ID) => {
                let is_some: bool = packet.read_state();

                if is_some != self.is_some.v() {
                    self.is_some.apply_state(is_some);
                    if is_some {
                        self.item.apply_state(S::default());
                    }
                }
            },
            Some(_) => Err(packet.error(depth, "unknown id"))?,
            None => {
                let option_state: Option<S> = packet.read_state();

                self.is_some.apply_state(option_state.is_some());

                if let Some(state) = option_state {
                    self.item.apply_state(state);
                }
            },
        })        
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

    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<OptionMessage<S>, PacketError> {
        match packet.get_id(depth) {
            Some(ITEM_ID) => {
                if self.is_some.v() {
                    Ok(OptionMessage::item(Some(self.item.apply_export(depth+1, packet)?)))
                } else {
                    Ok(OptionMessage::item(None))
                }
            },
            Some(IS_SOME_ID) => {
                let is_some: bool = packet.read_state();

                if is_some != self.is_some.v() {
                    self.is_some.apply_state(is_some);
                    if is_some {
                        self.item.apply_state(S::default());
                    }
                }
                
                Ok(OptionMessage::IsSome(is_some))
            },
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => {
                let option_state: Option<S> = packet.read_state();
                
                self.is_some.apply_state(option_state.is_some());

                if let Some(state) = &option_state {
                    self.item.apply_state(state.clone());
                }

                Ok(OptionMessage::State(option_state))
            },
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

impl<S: State> Node<Option<S>> for OptionNode<S> 
where Option<S>: State<Message = OptionMessage<S>, Consensus = OptionConsensus<S>> {    
    fn new_from(
        consensus: &OptionConsensus<S>,
        reporter: &Reporter,
    ) -> Self {
        Self { 
            key: consensus.key.clone(), 
            reporter: reporter.clone(), 
            is_some: Node::new_from(&consensus.is_some, reporter),
            item: Node::new_from(&consensus.item, reporter),
        }
    }    

    fn clone_state(&self) -> Option<S> { 
        match self.is_some.v() {
            true => Some(self.item.clone_state()),
            false => None,
        }
    }
}

impl<S: State> Emitter<Option<S>> for OptionNode<S> 
where Option<S>: State {      
    fn emit(&self, state: Option<S>) {
        self.reporter.report(&self.key, state)
    }    

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = Option<S>> + Send {
        self.reporter.report_future(&self.key, future)
    }
}