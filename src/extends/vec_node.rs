use std::{future::Future, ops::Index, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use bases::{Node, NodeId, NodeKey, Packet, PacketError, Reporter};
use crate::*;

const PUSH_ID: NodeId = NodeId::MAX - 1;
const POP_ID: NodeId = NodeId::MAX - 2;

#[derive(Debug,Clone)]
pub enum VecMessage<S: State> {
    #[allow(non_camel_case_types)] item((NodeId, S::Message)),
    Push(S),
    Pop(()),
    State(Vec<S>),
}

#[derive(Debug, Clone)]
pub struct VecConsensus<S: State> {     
    key: NodeKey,
    push: NodeKey,
    pop: NodeKey, 
    len: Arc<RwLock<NodeId>>,
    items: Arc<RwLock<Vec<S::Consensus>>>,
}

#[derive(Debug, Clone)]
pub struct VecNode<S: State> {
    key: NodeKey,
    push: NodeKey,
    pop: NodeKey, 
    reporter: Reporter,
    len: Arc<RwLock<NodeId>>,
    item_consensuses: Arc<RwLock<Vec<S::Consensus>>>,
    items: Arc<RwLock<Vec<Arc<S::Node>>>>,
}

pub struct VecNodeItems<'a, S:State> {
    index: usize,
    len: RwLockReadGuard<'a, NodeId>,
    items: RwLockReadGuard<'a, Vec<Arc<S::Node>>>,
}

impl<S: State> State for Vec<S> {
    type Message = VecMessage<S>;
    type Consensus = VecConsensus<S>;
    type Node = VecNode<S>; 

    fn apply(
        &mut self, 
        depth: usize, 
        packet: Packet,
    ) -> Result<(), PacketError>  {
        match packet.get_id(depth) {
            Some(PUSH_ID) => Ok(self.push(packet.read_state())),
            Some(POP_ID) => Ok({ self.pop(); }),
            Some(id) if (id as usize) < self.len() => self[id as usize].apply(depth + 1, packet),
            Some(_) => Err(packet.error(depth, "unknown id")),
            None => Ok(*self = packet.read_state()),
        }
    }    

    fn apply_message(
        &mut self,  
        message: Self::Message,
    ) {
        match message {
            Self::Message::Push(item) => self.push(item),
            Self::Message::Pop(()) => { self.pop(); },
            Self::Message::item((id, message)) => self[id as usize].apply_message(message),
            Self::Message::State(state) => *self = state,
        }
    }
}

impl<S: State> Message for VecMessage<S> {
    fn from_packet(
        depth: usize,
        packet: &Packet,
    ) -> Result<Self, PacketError> {
        match packet.get_id(depth) {
            Some(PUSH_ID) => Ok(Self::Push(packet.read_state())),
            Some(POP_ID) => Ok(Self::Pop(())),
            Some(id) => Ok(Self::item((id, S::Message::from_packet(depth + 1, packet)?))),
            None => Ok(Self::State(packet.read_state())),
        }
    }
}

impl<S: State> VecConsensus<S> {  
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

    fn items_read(&self) -> (RwLockReadGuard<NodeId>, RwLockReadGuard<Vec<S::Consensus>>) { 
        (
            self.len.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
            self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
        )        
    }

    fn items_write(&mut self) -> (&NodeKey, RwLockWriteGuard<NodeId>, RwLockWriteGuard<Vec<S::Consensus>>) { 
        (
            &self.key,
            self.len.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
            self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key)),
        ) 
    }

    fn push_inner(
        key: &NodeKey,
        len: &mut RwLockWriteGuard<NodeId>, 
        items: &mut RwLockWriteGuard<Vec<S::Consensus>>,
        item: S,
    ) {
        if (**len as usize) < items.len() {
            items[**len as usize].apply_state(item);
        } else {
            let mut consensus: S::Consensus = Consensus::new(
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

impl<S: State> Default for VecConsensus<S> 
where Vec<S>: State<Message = VecMessage<S>, Consensus = Self, Node = VecNode<S>> {      
    fn default() -> Self { Self::new(vec![], None) }
}

impl<S: State> Consensus<Vec<S>> for VecConsensus<S> 
where Vec<S>: State<Message = VecMessage<S>, Consensus = Self, Node = VecNode<S>> {    
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
    
    fn new_node(&self, reporter: &Reporter) -> VecNode<S> {
        Node::new_from(self, reporter)
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

    fn apply(&mut self, depth: usize, packet: &Packet) -> Result<(), PacketError> {
        match packet.get_id(depth) {
            Some(PUSH_ID) => Ok({
                self.push(packet.read_state());
            }),
            Some(POP_ID) => Ok({
                self.pop();
            }),
            Some(index) => {
                let (_, len, mut items) = self.items_write();
                if index < *len {
                    items[index as usize].apply(depth+1, packet)
                } else {
                    Err(packet.error(depth, "index out of range"))
                }
            },
            None => {
                let state: Vec<S> = packet.read_state();
                self.apply_state(state.clone());
                Ok(())
            },
        }        
    }

    fn apply_export(&mut self, depth: usize, packet: &Packet) -> Result<VecMessage<S>, PacketError> {
        match packet.get_id(depth) {
            Some(PUSH_ID) => Ok({
                let state: S = packet.read_state();
                self.push(state.clone());
                VecMessage::Push(state)     
            }),
            Some(POP_ID) => Ok({ 
                self.pop();
                VecMessage::Pop(())  
            }),
            Some(index) => Ok({
                let (_, len, mut items) = self.items_write();
                if index < *len {
                    VecMessage::item((
                        index, 
                        items[index as usize].apply_export(depth+1, packet)?,
                    ))
                } else {
                    Err(packet.error(depth, "index out of range"))?
                }
            }), 
            None => {
                let state: Vec<S> = packet.read_state();
                self.apply_state(state.clone());
                Ok(VecMessage::State(state))
            },
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

impl<S: State> VecNode<S> {      
    pub fn len(&self) -> NodeId { 
        *self.len_read()
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
        let mut item_key = self.key.to_vec();
        item_key.push(self.len() as NodeId);

        self.reporter.report(&self.push, item.clone());
        self.reporter.report(&item_key.into_boxed_slice(), item);
    }

    pub fn emit_pop(&self) {
        self.reporter.report(&self.pop, ())
    }
    
    pub fn emit_push_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = S> + Send,
    {
        self.reporter.report_future(&self.push, future)
    }
    
    pub fn emit_pop_future<Fu>(&self, future: Fu) 
    where 
    Fu: 'static + Future<Output = ()> + Send,
    {
        self.reporter.report_future(&self.pop, future)
    }

    fn len_read(&self) -> RwLockReadGuard<NodeId> {
        self.len.read()
        .unwrap_or_else(|err| panic!("{:?} {err}", self.key))
    }

    fn items_read(&self) -> (RwLockReadGuard<NodeId>, RwLockReadGuard<Vec<Arc<S::Node>>>) { 
        let len = self.len.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
        let items = self.items.read().unwrap_or_else(|err| panic!("{:?} {err}", self.key));

        if items.len() < *len as usize {
            drop(items);

            let item_consensuses = self.item_consensuses.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
            let mut items = self.items.write().unwrap_or_else(|err| panic!("{:?} {err}", self.key));
            let items_len = items.len();

            for i in items_len..(*len as usize) {
                let node = item_consensuses[i].new_node(&self.reporter);
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

impl<'a, S: State> Node<Vec<S>> for VecNode<S> 
where Vec<S>: State<Message = VecMessage<S>, Consensus = VecConsensus<S>> {    
    type State = Vec<S>;

    fn new_from(
        consensus: &VecConsensus<S>,
        reporter: &Reporter,
    ) -> Self {
        Self { 
            key: consensus.key.clone(),
            push: consensus.push.clone(),
            pop: consensus.pop.clone(),
            reporter: reporter.clone(),
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

impl<S: State> Emitter<Vec<S>> for VecNode<S> 
where Vec<S>: State {  
    fn emit(&self, state: Vec<S>) {
        self.reporter.report(&self.key, state)
    }    

    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = Vec<S>> + Send {
        self.reporter.report_future(&self.key, future)
    }
}

impl<'a, S: State> Iterator for VecNodeItems<'a, S> {
    type Item = Arc<S::Node>;
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