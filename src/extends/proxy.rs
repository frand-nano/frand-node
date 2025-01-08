use std::marker::PhantomData;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use bases::*;
use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy<A: Accessor>(PhantomData<A>);

#[derive(Debug, Clone)]
pub struct ProxyNode<A: Accessor<State = S>, S: State>{
    _phantom: PhantomData<A>,
    key: NodeKey,
    emitter: Option<Emitter>,
    subject: OnceCell<S::Node>,
}

impl<A: Accessor> Default for Proxy<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A: Accessor<State = S>, S: State> Accessor for Proxy<A> {
    type State = Self;
    type Message = ();
    type Node = ProxyNode<A, S>; 
}

impl<A: Accessor> Emitable for Proxy<A> {}

impl<A: Accessor> State for Proxy<A> {
    fn apply(
        &mut self,  
        _message: Self::Message,
    ) {}
}

impl<A: Accessor<State = S>, S: State> ProxyNode<A, S> {
    pub fn subject(&self) -> &S::Node { 
        self.subject.get()
        .unwrap_or_else(|| panic!("must set subject before use {:?}", self.key))
    }

    pub fn set_subject(&mut self, subject: &S::Node) { 
        self.subject.set(subject.clone())
        .unwrap_or_else(|err| panic!("must set subject before use {:?} {:?}", self.key, err))
    }
}

impl<A: Accessor<State = S>, S: State> Default for ProxyNode<A, S> {
    fn default() -> Self { Self::new(vec![], None, None) }
}

impl<A: Accessor<State = S>, S: State> Accessor for ProxyNode<A, S> {
    type State = Proxy<A>;
    type Message = ();    
    type Node = Self;
}

impl<A: Accessor<State = S>, S: State> Fallback for ProxyNode<A, S> 
where Proxy<A>: State<Message = ()> {    
    fn fallback(&self, _message: Self::Message, _delta: Option<f32>) {}
}

impl<A: Accessor<State = S>, S: State> System for ProxyNode<A, S> 
where Proxy<A>: State<Message = ()> {    
    fn handle(&self, _message: Self::Message, _delta: Option<f32>) {}
}

impl<A: Accessor<State = S>, S: State> Node<Proxy<A>> for ProxyNode<A, S> 
where Proxy<A>: State<Message = ()> {    
    fn key(&self) -> &NodeKey { &self.key }
    fn emitter(&self) -> Option<&Emitter> { self.emitter.as_ref() }
    fn clone_state(&self) -> Proxy<A> { Default::default() }
}

impl<A: Accessor<State = S>, S: State> NewNode<Proxy<A>> for ProxyNode<A, S> 
where Proxy<A>: State<Message = ()> {    
    fn new(
        mut key: Vec<NodeId>,
        id: Option<NodeId>,
        emitter: Option<&Emitter>,
    ) -> Self {        
        if let Some(id) = id { key.push(id); }
        
        Self { 
            _phantom: Default::default(),
            key: key.clone().into_boxed_slice(),   
            emitter: emitter.cloned(),
            subject: OnceCell::new(),
        }
    }
}

impl<A: Accessor<State = S>, S: State> Consensus<Proxy<A>> for ProxyNode<A, S> 
where Proxy<A>: State<Message = ()> {     
    fn new_from(
        node: &Self,
        emitter: Option<&Emitter>,
    ) -> Self {
        Self {
            _phantom: Default::default(),
            key: node.key.clone(),
            emitter: emitter.cloned(),
            subject: node.subject.clone(),
        }
    }

    fn set_emitter(&mut self, emitter: Option<&Emitter>) { self.emitter = emitter.cloned() }
    fn apply(&self, _message: ()) {}
    fn apply_state(&self, _state: Proxy<A>) {}
}