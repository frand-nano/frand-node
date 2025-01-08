use std::{future::Future, ops::Index, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use bases::*;
use crate::*;

const PUSH_ID: NodeId = NodeId::MAX - 1;
const POP_ID: NodeId = NodeId::MAX - 2;

pub trait VecConsensus<S: State> 
where S::Node: Consensus<S> {
    fn push(&self, item: S);
    fn pop(&self);

    fn items_write(&self) -> (&NodeKey, RwLockWriteGuard<NodeId>, RwLockWriteGuard<Vec<S::Node>>);
    fn consensus_items_write(&self) -> (&NodeKey, RwLockWriteGuard<NodeId>, RwLockWriteGuard<Vec<S::Node>>);

    fn push_inner(
        key: &NodeKey,
        consensus_emitter: Option<&Emitter>,
        len: &mut RwLockWriteGuard<NodeId>, 
        consensus_items: &mut RwLockWriteGuard<Vec<S::Node>>,
        item: S,
    ) {
        if (**len as usize) < consensus_items.len() {
            consensus_items[**len as usize].apply_state(item);
        } else {
            let consensus: S::Node = NewNode::new(
                key.to_vec(), 
                Some(**len),
                consensus_emitter,
            );
    
            consensus.apply_state(item);
    
            consensus_items.push(consensus);
        }

        **len = len.saturating_add(1);
    }

    fn pop_inner(
        len: &mut RwLockWriteGuard<NodeId>, 
    ) {
        **len = len.saturating_sub(1);
    }
}

#[derive(Debug, Clone)]
pub enum VecMessage<S: State> {
    Item((NodeId, S::Message)),
    Push(S),
    Pop(()),
    State(Vec<S>),
}

#[derive(Debug, Clone)]
pub struct VecNode<S: State> {
    key: NodeKey,
    push: NodeKey,
    pop: NodeKey, 
    emitter: Option<Emitter>,
    consensus_emitter: Option<Emitter>,
    len: Arc<RwLock<NodeId>>,
    consensus_items: Arc<RwLock<Vec<S::Node>>>,
    items: Arc<RwLock<Vec<S::Node>>>,
}

pub struct VecNodeItems<'a, S: State> {
    index: usize,
    len: RwLockReadGuard<'a, NodeId>,
    items: RwLockReadGuard<'a, Vec<S::Node>>,
}

impl<S: State> Accessor for Vec<S>
where S::Node: Consensus<S> {
    type State = Self;
    type Message = VecMessage<S>;
    type Node = VecNode<S>; 
}

impl<S: State> Emitable for Vec<S> {}

impl<S: State> State for Vec<S> 
where S::Node: Consensus<S> {
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
        packet: &PacketMessage, 
        depth: usize, 
    ) -> Result<Self, MessageError> {                             
        match packet.get_id(depth) {
            Some(PUSH_ID) => Ok(Self::Push(unsafe { 
                State::from_emitable(packet.payload()) 
            })),
            Some(POP_ID) => Ok(Self::Pop(())),
            Some(id) => Ok(Self::Item(
                (id, S::Message::from_packet_message(packet, depth + 1)?)
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
            Some(PUSH_ID) => Ok(Self::Push(
                packet.read_state()
            )),
            Some(POP_ID) => Ok(Self::Pop(())),
            Some(id) => Ok(Self::Item((id, S::Message::from_packet(packet, depth + 1)?))),
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
            Self::Item((_, message)) => message.to_packet(header),
            Self::Push(item) => Ok(Packet::new(header.clone(), item)),
            Self::Pop(()) => Ok(Packet::new(header.clone(), &())),
            Self::State(state) => Ok(Packet::new(header.clone(), state)),
        }
    }
}

impl<S: State> VecNode<S> 
where S::Node: Consensus<S> {  
    pub fn len(&self) -> NodeId { 
        *self.len.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
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

    fn items_read(&self) -> (RwLockReadGuard<NodeId>, RwLockReadGuard<Vec<S::Node>>) { 
        let len = self.len.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items_len = items.len();
        
        if items_len < *len as usize {
            drop(items);
            let mut items = self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            let item_consensuses = self.consensus_items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            for i in items_len..(*len as usize) {
                let node: S::Node = Consensus::new_from(&item_consensuses[i], self.emitter.as_ref());
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
    fn default() -> Self { Self::new(vec![], None, None) }
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
            Item((id, message)) if id < self.len() => {
                self.items()[id as usize].handle(message, delta)
            },
            Item(_) => (),
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
    fn key(&self) -> &NodeKey { &self.key }
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
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
        emitter: Option<&Emitter>,
    ) -> Self {        
        if let Some(id) = id { key.push(id); }
        
        let mut push = key.clone();
        push.push(PUSH_ID);
        
        let mut pop = key.clone();
        pop.push(POP_ID);

        let items: Arc<RwLock<Vec<S::Node>>> = Default::default();

        Self { 
            key: key.clone().into_boxed_slice(),   
            push: push.into_boxed_slice(),
            pop: pop.into_boxed_slice(),
            emitter: emitter.cloned(),
            consensus_emitter: emitter.cloned(),
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
        emitter: Option<&Emitter>,
    ) -> Self {
        Self {
            key: node.key.clone(),
            push: node.push.clone(),
            pop: node.pop.clone(),
            emitter: emitter.cloned(),
            consensus_emitter: node.consensus_emitter.clone().or(emitter.cloned()),
            len: node.len.clone(),
            consensus_items: node.consensus_items.clone(),
            items: Default::default(),
        }
    }

    fn set_emitter(&mut self, emitter: Option<&Emitter>) { 
        self.consensus_emitter = self.consensus_emitter.clone().or(emitter.cloned()); 
        self.emitter = emitter.cloned(); 

        let (_, _len, mut items) = self.items_write();
        for item in items.iter_mut() {
            item.set_emitter(emitter);
        }
    }

    fn apply(&self, message: VecMessage<S>) {
        match message {
            VecMessage::Push(item) => self.push(item),
            VecMessage::Pop(()) => self.pop(),
            VecMessage::Item((index, item)) => {
                let (_, _len, items) = self.items_write();
                items[index as usize].apply(item)
            },
            VecMessage::State(state) => self.apply_state(state),
        }      
    }

    fn apply_state(&self, state: Vec<S>) {
        let consensus_emitter = self.consensus_emitter.clone();
        let (key, mut len_write, mut consensus_items) = self.consensus_items_write();
        let len = *len_write as usize;

        if len < state.len() {
            for _ in len..state.len() {
                Self::push_inner(
                    key, 
                    consensus_emitter.as_ref(), 
                    &mut len_write, 
                    &mut consensus_items, 
                    S::default(),
                );
            }
        } else if state.len() < len {
            for _ in state.len()..len {
                Self::pop_inner(&mut len_write);
            }
        }
        
        consensus_items.iter_mut()
        .zip(state.into_iter())
        .for_each(|(item, state)| item.apply_state(state)); 
    }
}

impl<S: State> VecConsensus<S> for VecNode<S> 
where S::Node: Consensus<S> {
    fn push(&self, item: S) {
        let consensus_emitter = self.consensus_emitter.clone();
        let (key, mut len, mut consensus_items) = self.consensus_items_write();
        Self::push_inner(
            key, 
            consensus_emitter.as_ref(), 
            &mut len, 
            &mut consensus_items, 
            item,
        )
    }

    fn pop(&self) {
        Self::pop_inner(
            &mut self.len.write()
            .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
        )
    }

    fn items_write(&self) -> (&NodeKey, RwLockWriteGuard<NodeId>, RwLockWriteGuard<Vec<S::Node>>) {
        let len = self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let mut items = self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items_len = items.len();
        
        if items_len < *len as usize {
            let item_consensuses = self.consensus_items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            for i in items_len..(*len as usize) {
                let node: S::Node = Consensus::new_from(&item_consensuses[i], self.emitter.as_ref());
                items.push(node);
            }

            (&self.key, len, items)
        } else {
            (&self.key, len, items)
        }  
    }

    fn consensus_items_write(&self) -> (&NodeKey, RwLockWriteGuard<NodeId>, RwLockWriteGuard<Vec<S::Node>>) { 
        (
            &self.key,
            self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
            self.consensus_items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
        ) 
    }
}

impl<'a, S: State> Iterator for VecNodeItems<'a, S> {
    type Item = S::Node;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index < *self.len as usize {
            self.index += 1;
            Some(self.items[index].clone())
        } else {
            None
        }
    }
}

impl<'a, S: State> Index<usize> for VecNodeItems<'a, S> {
    type Output = S::Node;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}