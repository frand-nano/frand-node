use std::{future::Future, ops::Index, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use bases::*;
use crate::*;

const PUSH_ID: NodeId = NodeId::MAX - 1;
const POP_ID: NodeId = NodeId::MAX - 2;

#[derive(Debug, Clone)]
pub enum VecMessage<S: State> {
    #[allow(non_camel_case_types)] item((NodeId, S::Message)),
    Push(S),
    Pop(()),
    State(Vec<S>),
}

#[derive(Debug, Clone)]
pub struct VecConsensus<M: Message, S: State> {     
    key: NodeKey,
    push: NodeKey,
    pop: NodeKey, 
    len: Arc<RwLock<NodeId>>,
    items: Arc<RwLock<Vec<S::Consensus<M>>>>,
}

#[derive(Debug, Clone)]
pub struct VecNode<M: Message, S: State> {
    key: NodeKey,
    push: NodeKey,
    pop: NodeKey, 
    callback: Callback<M>,
    future_callback: FutureCallback<M>,
    len: Arc<RwLock<NodeId>>,
    item_consensuses: Arc<RwLock<Vec<S::Consensus<M>>>>,
    items: Arc<RwLock<Vec<Arc<S::Node<M>>>>>,
}

pub struct VecNodeItems<'a, M: Message, S: State> {
    index: usize,
    len: RwLockReadGuard<'a, NodeId>,
    items: RwLockReadGuard<'a, Vec<Arc<S::Node<M>>>>,
}

impl<S: State> State for Vec<S> {
    type Message = VecMessage<S>;
    type Consensus<M: Message> = VecConsensus<M, S>;
    type Node<M: Message> = VecNode<M, S>; 

    fn apply(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::Push(item) => self.push(item),
            Self::Message::Pop(()) => { self.pop(); },
            Self::Message::item((id, message)) => self[id as usize].apply(message),
            Self::Message::State(state) => *self = state,
        }
    }
}

impl<S: State> Message for VecMessage<S> {
    fn from_state<S2: State>(
        header: &Header, 
        depth: usize, 
        state: S2,
    ) -> Result<Self, MessageError> {                    
        match header.get(depth).copied() {
            Some(PUSH_ID) => Ok(Self::Push(
                unsafe { Self::cast_state(state) }
            )),
            Some(POP_ID) => Ok(Self::Pop(())),
            Some(id) => Ok(Self::item((id, S::Message::from_state(header, depth + 1, state)?))),
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
            Some(PUSH_ID) => Ok(Self::Push(
                packet.read_state()
            )),
            Some(POP_ID) => Ok(Self::Pop(())),
            Some(id) => Ok(Self::item((id, S::Message::from_packet(packet, depth + 1)?))),
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
            Self::item((_, message)) => message.to_packet(header),
            Self::Push(item) => Ok(Packet::new(header.clone(), item)),
            Self::Pop(()) => Ok(Packet::new(header.clone(), &())),
            Self::State(state) => Ok(Packet::new(header.clone(), state)),
        }
    }
}

impl<M: Message, S: State> VecConsensus<M, S> {  
    pub fn push(&mut self, item: S) {
        let (key, mut len, mut items) = self.items_write();
        Self::push_inner(key, &mut len, &mut items, item)
    }

    pub fn pop(&mut self) {
        Self::pop_inner(&mut self.len_write())
    }

    fn len_write(&self) -> RwLockWriteGuard<NodeId> {
        self.len.write()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }

    fn items_read(&self) -> (RwLockReadGuard<NodeId>, RwLockReadGuard<Vec<S::Consensus<M>>>) { 
        (
            self.len.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
            self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
        )        
    }

    fn items_write(&mut self) -> (&NodeKey, RwLockWriteGuard<NodeId>, RwLockWriteGuard<Vec<S::Consensus<M>>>) { 
        (
            &self.key,
            self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
            self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
        ) 
    }

    fn push_inner(
        key: &NodeKey,
        len: &mut RwLockWriteGuard<NodeId>, 
        items: &mut RwLockWriteGuard<Vec<S::Consensus<M>>>,
        item: S,
    ) {
        if (**len as usize) < items.len() {
            items[**len as usize].apply_state(item);
        } else {
            let mut consensus: S::Consensus<M> = Consensus::new(
                key.to_vec(), 
                Some(**len),
            );
    
            consensus.apply_state(item);
    
            items.push(consensus);
        }

        **len = len.saturating_add(1);
    }

    fn pop_inner(
        len: &mut RwLockWriteGuard<NodeId>, 
    ) {
        **len = len.saturating_sub(1);
    }
}

impl<M: Message, S: State> Default for VecConsensus<M, S> 
where Vec<S>: State<Message = VecMessage<S>, Consensus<M> = Self, Node<M> = VecNode<M, S>> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<M: Message, S: State> Consensus<M, Vec<S>> for VecConsensus<M, S> 
where Vec<S>: State<Message = VecMessage<S>, Consensus<M> = Self, Node<M> = VecNode<M, S>> {    
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
    ) -> Self {        
        if let Some(id) = id { key.push(id); }
        
        let mut push = key.clone();
        push.push(PUSH_ID);
        
        let mut pop = key.clone();
        pop.push(POP_ID);

        Self { 
            key: key.clone().into_boxed_slice(),   
            push: push.into_boxed_slice(),
            pop: pop.into_boxed_slice(),
            len: Default::default(),
            items: Default::default(),
        }
    }
    
    fn new_node(
        &self, 
        callback: &Callback<M>, 
        future_callback: &FutureCallback<M>,
    ) -> VecNode<M, S> {
        Node::new_from(self, callback, future_callback)
    }
             
    fn clone_state(&self) -> Vec<S> { 
        let (len, items) = self.items_read();
        let items_len = *len as usize;

        items.iter().enumerate()
        .filter_map(|(index, item)| {
            (index < items_len)
            .then(|| item.clone_state())
        })
        .collect()
    }

    fn apply(&mut self, message: VecMessage<S>) {
        match message {
            VecMessage::Push(item) => self.push(item),
            VecMessage::Pop(()) => self.pop(),
            VecMessage::item((index, item)) => {
                let (_, _len, mut items) = self.items_write();
                items[index as usize].apply(item)
            },
            VecMessage::State(state) => self.apply_state(state),
        }      
    }

    fn apply_state(&mut self, state: Vec<S>) {
        let (key, mut len, mut items) = self.items_write();
        let items_len = *len as usize;


        if items_len < state.len() {
            for _ in items_len..state.len() {
                Self::push_inner(key, &mut len, &mut items, S::default());
            }
        } else if state.len() < items_len {
            for _ in state.len()..items_len {
                Self::pop_inner(&mut len);
            }
        }
        
        items.iter_mut()
        .zip(state.into_iter())
        .for_each(|(item, state)| item.apply_state(state)); 
    }
}

impl<M: Message, S: State> VecNode<M, S> {      
    pub fn len(&self) -> NodeId { 
        *self.len_read()
    }

    pub fn items<'a>(&'a self) -> VecNodeItems<'a, M, S> {
        let (len, items) = self.items_read();

        VecNodeItems {
            index: 0,
            len,
            items,
        }
    }

    pub fn emit_push(&self, item: S) {        
        let mut item_key = self.key.to_vec();
        item_key.push(self.len() as NodeId);

        self.callback.emit(self.push.clone(), item.clone());
        self.callback.emit(item_key.into_boxed_slice(), item.clone());
    }

    pub fn emit_pop(&self) {
        self.callback.emit(self.pop.clone(), ());
    }
    
    pub fn emit_push_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = S> + Send,
    {
        self.future_callback.emit(self.push.clone(), future);
    }
    
    pub fn emit_pop_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = ()> + Send,
    {
        self.future_callback.emit(self.pop.clone(), future);
    }

    fn len_read(&self) -> RwLockReadGuard<NodeId> {
        self.len.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }

    fn items_read(&self) -> (RwLockReadGuard<NodeId>, RwLockReadGuard<Vec<Arc<S::Node<M>>>>) { 
        let len = self.len.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        if items.len() < *len as usize {
            drop(items);

            let item_consensuses = self.item_consensuses.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
            let mut items = self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
            let items_len = items.len();

            for i in items_len..(*len as usize) {
                let node = item_consensuses[i].new_node(&self.callback, &self.future_callback);
                items.push(Arc::new(node));
            }

            drop(items);

            let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

            (len, items)
        } else {
            (len, items)
        } 
    }
}

impl<'a, M: Message, S: State> Node<M, Vec<S>> for VecNode<M, S> 
where Vec<S>: State<Message = VecMessage<S>, Consensus<M> = VecConsensus<M, S>> {    
    type State = Vec<S>;

    fn key(&self) -> &NodeKey { &self.key }

    fn new_from(
        consensus: &VecConsensus<M, S>,
        callback: &Callback<M>,
        future_callback: &FutureCallback<M>,
    ) -> Self {
        Self { 
            key: consensus.key.clone(),
            push: consensus.push.clone(),
            pop: consensus.pop.clone(),
            callback: callback.clone(), 
            future_callback: future_callback.clone(), 
            len: consensus.len.clone(),
            item_consensuses: consensus.items.clone(),
            items: Default::default(),
        }
    }        

    fn clone_state(&self) -> Vec<S> { 
        self.items()
        .map(|item| item.clone_state())
        .collect()
    }
}

impl<M: Message, S: State> Emitter<M, Vec<S>> for VecNode<M, S> 
where Vec<S>: State {  
    fn emit(&self, state: Vec<S>) {
        self.callback.emit(self.key.clone(), state)
    }    

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = Vec<S>> + Send {
        self.future_callback.emit(self.key.clone(), future)
    }
}

impl<'a, M: Message, S: State> Iterator for VecNodeItems<'a, M, S> {
    type Item = Arc<S::Node<M>>;
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

impl<'a, M: Message, S: State> Index<usize> for VecNodeItems<'a, M, S> {
    type Output = S::Node<M>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}