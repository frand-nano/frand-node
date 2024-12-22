use std::{future::Future, sync::Arc};
use crossbeam::channel::Sender;
use futures::{future::BoxFuture, FutureExt};
use crate::bases::Packet;
use super::*;

pub type EmitableFuture = (NodeKey, BoxFuture<'static, Box<dyn Emitable>>);

#[derive(Clone)]
pub enum Reporter {
    Callback(Arc<dyn Fn(Packet) + Send + Sync>),
    Sender(Sender<Packet>),
    FutureCallback(Arc<dyn Fn(EmitableFuture) + Send + Sync>),
    None,
}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reporter::Callback(_) => {
                write!(f, "Reporter::Callback(Arc<dyn Fn(Packet) + Send + Sync>)")
            },
            Reporter::Sender(sender) => {
                write!(f, "Reporter::Sender({:#?})", sender)
            },
            Reporter::FutureCallback(_) => {
                write!(f, "Reporter::FutureCallback(Arc<dyn Fn((NodeKey, BoxFuture<dyn Emitable>)) + Send + Sync>)")
            },
            Reporter::None => write!(f, "Reporter::None"),
        }
    }
}

impl Reporter {
    pub fn new_callback<F>(callback: F) -> Self 
    where F: 'static + Fn(Packet) + Send + Sync { 
        Self::Callback(Arc::new(callback)) 
    }

    pub fn new_sender(sender: Sender<Packet>) -> Self { 
        Self::Sender(sender) 
    }
    
    pub fn new_future_callback<F>(callback: F) -> Self 
    where F: 'static + Fn(EmitableFuture) + Send + Sync { 
        Self::FutureCallback(Arc::new(callback)) 
    }

    pub fn report<E: 'static + Emitable>(
        &self, 
        node_key: &NodeKey, 
        emitable: E,
    ) {
        match self {
            Reporter::Callback(callback) => { 
                callback(emitable.into_packet(node_key)); 
            },
            Reporter::Sender(sender) => { 
                sender.send(emitable.into_packet(node_key)).ok(); 
            },
            Reporter::FutureCallback(callback) => { 
                callback((node_key.to_owned(), async { 
                    Box::new(emitable) as Box<dyn Emitable> 
                }.boxed())); 
            },
            Reporter::None => {
                
            },
        }
    }

    pub fn report_future<Fu, E>(
        &self, 
        node_key: &NodeKey, 
        future: Fu,
    ) 
    where 
    Fu: 'static + Future<Output = E> + Send,
    E: 'static + Emitable + Sized,
    {
        match self {
            Reporter::Callback(_) => { unimplemented!(); },
            Reporter::Sender(_) => { unimplemented!(); },
            Reporter::FutureCallback(callback) => { 
                callback((node_key.to_owned(), async {
                    Box::new(future.await) as Box<dyn Emitable>
                }.boxed())); 
            },
            Reporter::None => {},
        }
    }
}