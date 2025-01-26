use std::{future::Future, sync::{Arc, RwLock, RwLockReadGuard}};
use bases::*;
use crate::*;

const PUSH_DELTA: IdDelta = 1;
const POP_DELTA: IdDelta = 2;
const ITEM_DELTA: IdDelta = 3;
const NODE_SIZE: IdDelta = 3;

#[derive(Debug, Clone)]
pub enum VecMessage<A: Accessor> {
    Push(A::State),
    Pop(()),
    Item((Index, A::Message)),
    State(Vec<A::State>),
}

#[derive(Debug, Clone)]
pub struct VecNode<A: Accessor> {
    depth: Depth,
    key: Key,
    push_key: Key,
    pop_key: Key,
    emitter: Option<Emitter>,
    consensus_emitter: Option<Emitter>,
    len: Arc<RwLock<usize>>,
    consensus_items: Arc<RwLock<Vec<A::Node>>>,
    items: Arc<RwLock<Vec<A::Node>>>,
}

pub struct VecNodeItems<'a, S: State> {
    index: usize,
    len: RwLockReadGuard<'a, usize>,
    items: RwLockReadGuard<'a, Vec<S::Node>>,
}

impl<A: Accessor> Accessor for Vec<A>
where A::Node: Consensus<A::State> {
    type State = Vec<A::State>;
    type Message = VecMessage<A::State>;
    type Node = VecNode<A::State>; 
}

impl<A: Accessor> Emitable for Vec<A> {}

impl<S: State> State for Vec<S> 
where S::Node: Consensus<S> {
    const NODE_SIZE: IdDelta = NODE_SIZE + S::NODE_SIZE; 

    fn apply(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::Push(item) => self.push(item),
            Self::Message::Pop(()) => { self.pop(); },
            Self::Message::Item((index, message)) => self[index as usize].apply(message),
            Self::Message::State(state) => *self = state,
        }
    }    
}

impl<S: State> Message for VecMessage<S> 
where S::Node: Consensus<S> {
    fn from_packet_message(
        parent_key: Key,
        depth: Depth,
        packet: &PacketMessage, 
    ) -> Result<Self, MessageError> {                    
        match packet.key().id() - parent_key.id() {      
            0 => Ok(Self::State(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
            PUSH_DELTA => Ok(Self::Push(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
            POP_DELTA => Ok(Self::Pop(())),
            _ => {
                Ok(Self::Item(
                    (packet.key().index(depth), S::Message::from_packet_message(
                        parent_key + ITEM_DELTA,
                        depth + 1,
                        packet, 
                    )?)
                ))
            },      
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
            PUSH_DELTA => Ok(Self::Push(
                packet.read_state()
            )),
            POP_DELTA => Ok(Self::Pop(())),
            _ => {                
                Ok(Self::Item(
                    (packet.key().index(depth), S::Message::from_packet(
                        parent_key + ITEM_DELTA,
                        depth + 1,
                        packet, 
                    )?)
                ))
            },
        }
    }

    fn to_packet(
        &self,
        key: Key, 
    ) -> Result<Packet, MessageError> {
        match self {
            Self::Item((_, message)) => message.to_packet(key),
            Self::Push(item) => Ok(Packet::new(key, item)),
            Self::Pop(()) => Ok(Packet::new(key, &())),
            Self::State(state) => Ok(Packet::new(key, state)),
        }
    }
}

impl<S: State> VecNode<S> 
where S::Node: Consensus<S> {  
    pub fn len(&self) -> usize { 
        *self.len.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }

    pub fn item(&self, index: Index) -> S::Node {
        let index = index as usize;

        self.extend_consensus_items_capacity(index + 1);
        self.extend_items_capacity(index + 1);

        let (_len, items) = self.read_items();

        items[index].clone()
    }

    pub fn active_item(&self, index: Index) -> Option<S::Node> {
        let (len, items) = self.read_items();
        let index = index as usize;

        if index < *len {
            Some(items[index].clone())
        } else {
            None
        }
    }

    pub fn items<'a>(&'a self) -> VecNodeItems<'a, S> {
        let (len, items) = self.read_items();

        VecNodeItems {
            index: 0,
            len,
            items,
        }
    }

    pub fn emit_push(&self, item: S) {        
        if let Some(emitter) = &self.emitter {
            emitter.emit(self.push_key.clone(), item.clone());
        }
    }

    pub fn emit_pop(&self) {   
        if let Some(emitter) = &self.emitter {
            emitter.emit(self.pop_key.clone(), ());
        }
    }
    
    pub fn emit_push_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = S> + Send,
    {
        if let Some(emitter) = &self.emitter {
            emitter.emit_future(self.push_key.clone(), future);
        }
    }
    
    pub fn emit_pop_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = ()> + Send,
    {
        if let Some(emitter) = &self.emitter {
            emitter.emit_future(self.pop_key.clone(), future);
        }
    }

    fn read_items(&self) -> (RwLockReadGuard<usize>, RwLockReadGuard<Vec<S::Node>>) { 
        let consensus_items_len = self.consensus_items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
        .len();

        self.extend_items_capacity(consensus_items_len);
        
        let len = self.len.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        let items = self.items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        (len, items)
    }
}

impl<S: State> Default for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{   
    fn default() -> Self { Self::new(
        Key::default(), 
        IdDelta::default(), 
        Depth::default(), 
        None,
    ) }
}

impl<S: State> Accessor for VecNode<S>
where S::Node: Consensus<S> {  
    type State = Vec<S>;
    type Message = VecMessage<S>;    
    type Node = Self;
}

impl<S: State> Fallback for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{   
    fn fallback(&self, message: VecMessage<S>, delta: Option<f32>) {
        use VecMessage::*;
        match message {
            Item((index, message)) => {
                self.item(index).handle(message, delta)
            },
            Push(_) => (),
            Pop(_) => (),
            State(_) => (),
        }
    }
}

impl<S: State> System for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{   
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        self.fallback(message, delta);
    }
}

impl<S: State> Node<Vec<S>> for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{           
    fn key(&self) -> Key { self.key }
    fn emitter(&self) -> Option<&Emitter> { self.emitter.as_ref() }  

    fn clone_state(&self) -> Vec<S> { 
        self.items()
        .map(|item| item.clone_state())
        .collect()
    }
}

impl<S: State> NewNode<Vec<S>> for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{   
    fn new(
        mut key: Key,
        id_delta: IdDelta,
        depth: Depth,
        emitter: Option<Emitter>,
    ) -> Self {        
        key = key + id_delta;

        let items: Arc<RwLock<Vec<S::Node>>> = Default::default();

        Self { 
            depth,
            key,   
            push_key: key + PUSH_DELTA,
            pop_key: key + POP_DELTA,
            emitter: emitter.clone(),
            consensus_emitter: emitter.clone(),
            len: Default::default(),
            consensus_items: items.clone(),
            items,
        }
    }
}

impl<S: State> Consensus<Vec<S>> for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{        
    fn new_from(
        node: &Self,
        emitter: Option<Emitter>,
    ) -> Self {
        Self {
            depth: node.depth,
            key: node.key,
            push_key: node.push_key.clone(),
            pop_key: node.pop_key.clone(),
            emitter: emitter.clone(),
            consensus_emitter: node.consensus_emitter.clone().or(emitter.clone()),
            len: node.len.clone(),
            consensus_items: node.consensus_items.clone(),
            items: Default::default(),
        }
    }

    fn set_emitter(&mut self, emitter: Option<Emitter>) { 
        self.consensus_emitter = self.consensus_emitter.clone().or(emitter.clone()); 
        self.emitter = emitter.clone(); 

        let _len = self.len.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        let mut items = self.items.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        for item in items.iter_mut() {
            item.set_emitter(emitter.clone());
        }
    }

    fn apply(&self, message: VecMessage<S>) {
        match message {
            VecMessage::Push(item) => {
                self.push(item.clone());
            },
            VecMessage::Pop(()) => {
                self.pop();
            },
            VecMessage::Item((index, message)) => {
                self.item(index).apply(message)
            },
            VecMessage::State(state) => {            
                self.apply_state(state.clone());
            },
        }      
    }

    fn apply_state(&self, state: Vec<S>) {
        self.apply_many(state);
    }
}

pub trait VecConsensus<S: State> 
where S::Node: Consensus<S> {
    fn push(&self, item: S) { self.push_many(vec![item]) }
    fn pop(&self) { self.pop_many(1) }
    fn push_many(&self, new_states: Vec<S>);
    fn pop_many(&self, count: usize);
    fn apply_many(&self, new_states: Vec<S>);
    fn set_len(&self, len: usize);
    fn extend_consensus_items_capacity(&self, len: usize);
    fn extend_items_capacity(&self, len: usize);
}

impl<S: State> VecConsensus<S> for VecNode<S> 
where S::Node: Consensus<S> {
    fn push_many(&self, new_states: Vec<S>) {
        let mut len = self.len.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        let new_len = len.saturating_add(new_states.len());

        self.extend_consensus_items_capacity(new_len);
        self.extend_items_capacity(new_len);

        let items = self.items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        for (item, state) in items.iter().skip(*len).zip(new_states) {
            item.emit(state);
        }

        *len = new_len;
    }

    fn pop_many(&self, count: usize) {
        let mut len = self.len.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        let new_len = len.saturating_sub(count);

        self.extend_items_capacity(*len);

        let items = self.items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        for index in new_len..*len {
            items[index].emit(Default::default());
        }

        *len = new_len;
    }

    fn apply_many(&self, new_states: Vec<S>) {
        let mut len = self.len.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        let new_len = new_states.len();

        self.extend_consensus_items_capacity(new_len);
        self.extend_items_capacity(new_len);

        let items = self.items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        for (item, state) in items.iter().zip(new_states) {
            item.emit(state);
        }

        for index in new_len..*len {
            items[index].emit(Default::default());
        }

        *len = new_len;
    }

    fn set_len(&self, new_len: usize) {
        self.extend_consensus_items_capacity(new_len);
        self.extend_items_capacity(new_len);

        *self.len.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key)) = new_len;
    }

    fn extend_consensus_items_capacity(&self, len: usize) {        
        let consensus_items_len = self.consensus_items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
        .len();   

        if consensus_items_len < len {
            let mut consensus_items = self.consensus_items.write()
            .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            for index in consensus_items_len..len {
                let mut key = self.key;
                key.set_index(self.depth, index as Index);

                consensus_items.push(NewNode::new(
                    key, 
                    ITEM_DELTA,
                    self.depth + 1,
                    self.consensus_emitter.clone(),
                ))
            }
        }   
    }

    fn extend_items_capacity(&self, len: usize) {        
        let items_len = self.items.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
        .len();   

        if items_len < len {
            let consensus_items = self.consensus_items.read()
            .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            let mut items = self.items.write()
            .unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            for index in items_len..len {
                items.push(Consensus::new_from(
                    &consensus_items[index], 
                    self.emitter.clone(),
                ))
            }            
        }
    }
}

impl<'a, S: State> Iterator for VecNodeItems<'a, S> {
    type Item = S::Node;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index < *self.len {
            self.index += 1;
            Some(self.items[index].clone())
        } else {
            None
        }
    }
}