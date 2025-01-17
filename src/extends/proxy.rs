use std::marker::PhantomData;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use bases::*;
use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy<A: Accessor, L: Accessor = (), S: Accessor = A> {
    _phantom_a: PhantomData<A>,
    _phantom_s: PhantomData<S>,
    locate: L::State,
}

#[derive(Debug, Clone)]
pub struct ProxyNode<A: Accessor, L: Accessor = (), S: Accessor = A>{
    _phantom: PhantomData<A>,
    subject: OnceCell<(A::Node, fn(&A::Node, L::State) -> Option<S::Node>)>,
    locate: L::Node,
}

impl<A: Accessor, L: Accessor, S: Accessor> Default for Proxy<A, L, S> {
    fn default() -> Self {
        Self { 
            _phantom_a: Default::default(), 
            _phantom_s: Default::default(), 
            locate: Default::default(), 
        }
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> Accessor for Proxy<A, L, S> {
    type State = Self;
    type Message = L::Message;
    type Node = ProxyNode<A, L, S>; 
}

impl<A: Accessor, L: Accessor, S: Accessor> Emitable for Proxy<A, L, S> {}

impl<A: Accessor, L: Accessor, S: Accessor> State for Proxy<A, L, S> {
    const NODE_SIZE: Index = <L::State as State>::NODE_SIZE; 

    fn apply(
        &mut self,  
        message: Self::Message,
    ) {
        self.locate.apply(message)
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> ProxyNode<A, L, S> {
    pub fn locate(&self) -> &L::Node { &self.locate }

    pub fn subject(&self) -> Option<S::Node> { 
        let (container, selector) = self.subject.get()
        .unwrap_or_else(|| panic!("must set subject before use {:?}", self.key()));

        selector(container, self.locate.clone_state())
    }

    pub fn set_subject(&mut self, container: &A::Node) 
    where A: Accessor<Node = S::Node> {
        self.subject.set((container.clone(), |c, _| Some(c.clone())))
        .unwrap_or_else(|err| panic!("subject already set {:?} {:?}", self.key(), err))
    }

    pub fn set_selector(
        &mut self, 
        container: &A::Node, 
        selector: fn(&A::Node, L::State) -> Option<S::Node>,
    ) {
        self.subject.set((container.clone(), selector))
        .unwrap_or_else(|err| panic!("selector already set {:?} {:?}", self.key(), err))
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> Default for ProxyNode<A, L, S> {
    fn default() -> Self { Self::new(Key::default(), 0, None) }
}

impl<A: Accessor, L: Accessor, S: Accessor> Accessor for ProxyNode<A, L, S> {
    type State = Proxy<A, L, S>;
    type Message = L::Message;    
    type Node = Self;
}

impl<A: Accessor, L: Accessor, S: Accessor> Fallback for ProxyNode<A, L, S> {    
    fn fallback(&self, message: Self::Message, delta: Option<f32>) {
        self.locate.handle(message, delta)
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> System for ProxyNode<A, L, S> {    
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        self.fallback(message, delta);
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> Node<Proxy<A, L, S>> for ProxyNode<A, L, S> {    
    fn key(&self) -> Key { self.locate.key() }
    fn emitter(&self) -> Option<&Emitter> { self.locate.emitter() }
    fn clone_state(&self) -> Proxy<A, L, S> { 
        Proxy { 
            _phantom_a: Default::default(), 
            _phantom_s: Default::default(), 
            locate: self.locate.clone_state(), 
        } 
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> NewNode<Proxy<A, L, S>> for ProxyNode<A, L, S> {    
    fn new(
        key: Key,
        index: Index,
        emitter: Option<Emitter>,
    ) -> Self {                
        Self { 
            _phantom: Default::default(),
            subject: OnceCell::new(),
            locate: NewNode::new(key, index, emitter.clone()),
        }
    }
}

impl<A: Accessor, L: Accessor, S: Accessor> Consensus<Proxy<A, L, S>> for ProxyNode<A, L, S> 
where 
L::Node: Consensus<L::State>,
Proxy<A, L, S>: State<Message = L::Message>,
{     
    fn new_from(
        node: &Self,
        emitter: Option<Emitter>,
    ) -> Self {
        Self {
            _phantom: Default::default(),
            subject: node.subject.clone(),
            locate: Consensus::new_from(&node.locate, emitter),
        }
    }

    fn set_emitter(&mut self, emitter: Option<Emitter>) { self.locate.set_emitter(emitter) }

    fn apply(&self, message: L::Message) {
        self.locate.apply(message)
    }

    fn apply_state(&self, state: Proxy<A, L, S>) {
        self.locate.apply_state(state.locate)
    }
}