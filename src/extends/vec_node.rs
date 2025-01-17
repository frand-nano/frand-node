use std::{future::Future, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use bases::*;
use crate::*;

const MAX_CAPACITY: Index = 100_000;

const PUSH_INDEX: Index = 1;
const POP_INDEX: Index = 2;
const ITEM_INDEX: Index = 3;

#[derive(Debug, Clone)]
pub enum VecMessage<A: Accessor> {
    Item((Index, A::Message)),
    Push(A::State),
    Pop(()),
    State(Vec<A::State>),
}

#[derive(Debug, Clone)]
pub struct VecNode<A: Accessor> {
    key: Key,
    push: Key,
    pop: Key, 
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

pub enum VecNodeItem<'a, A: Accessor> {
    Active(&'a A::Node),
    Inactive(&'a A::Node),
    None,
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
    const NODE_SIZE: Index = ITEM_INDEX + S::NODE_SIZE * MAX_CAPACITY; 

    fn apply(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::Push(item) => self.push(item),
            Self::Message::Pop(()) => { self.pop(); },
            Self::Message::Item((id, message)) => self[id as usize].apply(message),
            Self::Message::State(state) => *self = state,
        }
    }    
}

impl<S: State> Message for VecMessage<S> 
where S::Node: Consensus<S> {
    fn from_packet_message(
        parent_key: Key,
        packet: &PacketMessage, 
    ) -> Result<Self, MessageError> {                    
        match packet.key() - parent_key {      
            0 => Ok(Self::State(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
            PUSH_INDEX => Ok(Self::Push(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
            POP_INDEX => Ok(Self::Pop(())),
            index => {
                let index = (index - ITEM_INDEX) / S::NODE_SIZE;
                Ok(Self::Item(
                    (index, S::Message::from_packet_message(
                        parent_key + (ITEM_INDEX + index * S::NODE_SIZE),
                        packet, 
                    )?)
                ))
            },      
        }
    }

    fn from_packet(
        parent_key: Key,
        packet: &Packet, 
    ) -> Result<Self, PacketError> {       
        match packet.key() - parent_key {
            0 => Ok(Self::State(
                packet.read_state()
            )),
            PUSH_INDEX => Ok(Self::Push(
                packet.read_state()
            )),
            POP_INDEX => Ok(Self::Pop(())),
            index => {                
                let index = (index - ITEM_INDEX) / S::NODE_SIZE;
                Ok(Self::Item(
                    (index, S::Message::from_packet(
                        parent_key + (ITEM_INDEX + index * S::NODE_SIZE),
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

    pub fn item(&self, index: Index) -> Option<S::Node> {
        let (len, items) = self.items_read();
        match Self::item_inner(&len, &items, index as usize) {
            VecNodeItem::Active(item) => Some(item.clone()),
            VecNodeItem::Inactive(item) => Some(item.clone()),
            VecNodeItem::None => None,
        }    
    }

    pub fn active_item(&self, index: Index) -> Option<S::Node> {
        let (len, items) = self.items_read();
        match Self::item_inner(&len, &items, index as usize) {
            VecNodeItem::Active(item) => Some(item.clone()),
            VecNodeItem::Inactive(_) => None,
            VecNodeItem::None => None,
        }    
    }

    pub fn items<'a>(&'a self) -> VecNodeItems<'a, S> {
        let (len, items) = self.items_read();

        VecNodeItems {
            index: 0,
            len,
            items,
        }
    }

    pub fn emit_push(&self, item: S) {        
        if let Some(emitter) = &self.emitter {
            emitter.emit(self.push.clone(), item.clone());
        }
    }

    pub fn emit_pop(&self) {   
        if let Some(emitter) = &self.emitter {
            emitter.emit(self.pop.clone(), ());
        }
    }
    
    pub fn emit_push_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = S> + Send,
    {
        if let Some(emitter) = &self.emitter {
            emitter.emit_future(self.push.clone(), future);
        }
    }
    
    pub fn emit_pop_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = ()> + Send,
    {
        if let Some(emitter) = &self.emitter {
            emitter.emit_future(self.pop.clone(), future);
        }
    }

    fn items_read(&self) -> (RwLockReadGuard<usize>, RwLockReadGuard<Vec<S::Node>>) { 
        let len = self.len.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items_len = items.len();
        
        if items_len < *len {
            drop(items);
            let mut items = self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            let consensus_items = self.consensus_items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            for i in items_len..*len {
                let node: S::Node = Consensus::new_from(&consensus_items[i], self.emitter.clone());
                items.push(node);
            }

            drop(items);
            let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            (len, items)
        } else {
            (len, items)
        }      
    }
}

impl<S: State> Default for VecNode<S> 
where 
S::Node: Consensus<S>,
Vec<S>: State<Message = VecMessage<S>>, 
{   
    fn default() -> Self { Self::new(Key::default(), 0, None) }
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
                if let Some(item) = self.item(index) {
                    item.handle(message, delta)
                }
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
        index: Index,
        emitter: Option<Emitter>,
    ) -> Self {        
        key = key + index;

        let items: Arc<RwLock<Vec<S::Node>>> = Default::default();

        Self { 
            key,   
            push: key + PUSH_INDEX,
            pop: key + POP_INDEX,
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
            key: node.key,
            push: node.push.clone(),
            pop: node.pop.clone(),
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

        let (_, _len, mut items) = self.items_write();
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
                if let Some(item) = self.item(index) {
                    item.apply(message)
                }
            },
            VecMessage::State(state) => {            
                self.apply_state(state.clone());
                for (item, state) in self.items().zip(state) {
                    item.emit(state);
                }
            },
        }      
    }

    fn apply_state(&self, state: Vec<S>) {
        let consensus_emitter = self.consensus_emitter.clone();
        let (key, mut len_write, mut consensus_items) = self.consensus_items_write();
        let len = *len_write;

        if len < state.len() {

            for _ in len..state.len() {
                Self::push_inner(
                    key, 
                    consensus_emitter.clone(), 
                    &mut len_write, 
                    &mut consensus_items, 
                    S::default(),
                );
            }
        } else if state.len() < len {
            let items = self.items.read()
            .unwrap_or_else(|err| panic!("{:?} {err}", key));

            for _ in state.len()..len {
                Self::pop_inner(
                    &mut len_write,
                    &items, 
                );
            }
        }
        
        consensus_items.iter_mut()
        .zip(state.into_iter())
        .for_each(|(item, state)| item.apply_state(state)); 
    }
}

pub trait VecConsensus<S: State> 
where S::Node: Consensus<S> {
    fn push(&self, item: S);
    fn pop(&self);

    fn items_write(&self) -> (Key, RwLockWriteGuard<usize>, RwLockWriteGuard<Vec<S::Node>>);
    fn consensus_items_write(&self) -> (Key, RwLockWriteGuard<usize>, RwLockWriteGuard<Vec<S::Node>>);

    fn push_inner(
        key: Key,
        consensus_emitter: Option<Emitter>,
        len: &mut RwLockWriteGuard<usize>, 
        consensus_items: &mut RwLockWriteGuard<Vec<S::Node>>,
        item: S,
    ) {
        if **len < consensus_items.len() {
            consensus_items[**len].apply_state(item);
        } else {
            let consensus: S::Node = NewNode::new(
                key, 
                ITEM_INDEX + **len as Index * S::NODE_SIZE,
                consensus_emitter,
            );
    
            consensus.apply_state(item);
    
            consensus_items.push(consensus);
        }

        **len = len.saturating_add(1);
    }

    fn pop_inner(
        len: &mut RwLockWriteGuard<usize>, 
        items: &RwLockReadGuard<Vec<S::Node>>,
    ) {
        **len = len.saturating_sub(1);
        items[**len].emit(Default::default());
    }

    fn item_inner<'a>(
        len: &'a RwLockReadGuard<usize>, 
        items: &'a RwLockReadGuard<Vec<S::Node>>,
        index: usize,
    ) -> VecNodeItem<'a, S::Node> {
        if index < **len {
            VecNodeItem::Active(&items[index])
        } else {       
            match items.get(index) {
                Some(item) => VecNodeItem::Inactive(item),
                None => VecNodeItem::None,
            }
        }      
    }
}

impl<S: State> VecConsensus<S> for VecNode<S> 
where S::Node: Consensus<S> {
    fn push(&self, item: S) {
        let consensus_emitter = self.consensus_emitter.clone();
        let (key, mut len, mut consensus_items) = self.consensus_items_write();
        Self::push_inner(
            key, 
            consensus_emitter, 
            &mut len, 
            &mut consensus_items, 
            item,
        )
    }

    fn pop(&self) {
        let mut len = self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        Self::pop_inner(
            &mut len, 
            &items, 
        )
    }

    fn items_write(&self) -> (Key, RwLockWriteGuard<usize>, RwLockWriteGuard<Vec<S::Node>>) {
        let len = self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let mut items = self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items_len = items.len();
        
        if items_len < *len {
            let consensus_items = self.consensus_items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            for i in items_len..*len {
                let node: S::Node = Consensus::new_from(&consensus_items[i], self.emitter.clone());
                items.push(node);
            }

            (self.key, len, items)
        } else {
            (self.key, len, items)
        }  
    }

    fn consensus_items_write(&self) -> (Key, RwLockWriteGuard<usize>, RwLockWriteGuard<Vec<S::Node>>) { 
        (
            self.key,
            self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
            self.consensus_items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
        ) 
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