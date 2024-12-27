use std::{any::type_name_of_val, future::Future, sync::Arc};
use crossbeam::channel::Sender;
use futures::{future::BoxFuture, FutureExt};
use super::*;

pub type EmitableFuture<M> = (NodeKey, BoxFuture<'static, M>);

#[derive(Clone)]
pub enum Reporter<M: Message> {
    Callback(Arc<dyn Fn(PacketMessage<M>) + Send + Sync>),
    Sender(Sender<PacketMessage<M>>),
    FutureCallback(Arc<dyn Fn(EmitableFuture<M>) + Send + Sync>),
}

impl<M: Message> std::fmt::Debug for Reporter<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reporter::Callback(callback) => {
                write!(f, "Reporter::Callback({})", type_name_of_val(callback))
            },
            Reporter::Sender(sender) => {
                write!(f, "Reporter::Sender({:#?})", sender)
            },
            Reporter::FutureCallback(callback) => {
                write!(f, "Reporter::FutureCallback({})", type_name_of_val(callback))
            },
        }
    }
}

impl<M: Message> Reporter<M> {
    pub fn new_callback<F>(callback: F) -> Self 
    where F: 'static + Fn(PacketMessage<M>) + Send + Sync { 
        Self::Callback(Arc::new(callback)) 
    }

    pub fn new_sender(sender: Sender<PacketMessage<M>>) -> Self { 
        Self::Sender(sender) 
    }
    
    pub fn new_future_callback<F>(callback: F) -> Self 
    where F: 'static + Fn(EmitableFuture<M>) + Send + Sync { 
        Self::FutureCallback(Arc::new(callback)) 
    }

    pub fn report<S: State>(
        &self, 
        node_key: &NodeKey, 
        state: S,
    ) {
        match self {
            Reporter::Callback(callback) => { 
                callback(
                    PacketMessage {
                        header: node_key.clone(),
                        message: M::from_state(node_key, 0, state)
                        .unwrap_or_else(|err| panic!("{:?} {err}", node_key)),
                    }                    
                ); 
            },
            Reporter::Sender(sender) => {
                sender.send(
                    PacketMessage {
                        header: node_key.clone(),
                        message: M::from_state(node_key, 0, state)
                        .unwrap_or_else(|err| panic!("{:?} {err}", node_key)),
                    }     
                ).unwrap_or_else(|err| panic!("{:?} {err}", node_key)); 
            },
            Reporter::FutureCallback(callback) => { 
                let node_key = node_key.clone();
                callback((node_key.clone(), async move { 
                    M::from_state(&node_key, 0, state) 
                    .unwrap_or_else(|err| panic!("{:?} {err}", node_key))
                }.boxed())); 
            },
        }
    }

    pub fn report_future<S: State, Fu>(
        &self, 
        node_key: NodeKey, 
        future: Fu,
    ) where Fu: 'static + Future<Output = S> + Send {
        match self {
            Reporter::Callback(_) => { unimplemented!(); },
            Reporter::Sender(_) => { unimplemented!(); },
            Reporter::FutureCallback(callback) => { 
                callback((node_key.clone(), async move {
                    M::from_state(&node_key, 0, future.await)
                    .unwrap_or_else(|err| panic!("{:?} {err}", node_key))
                }.boxed())); 
            },
        }
    }
}