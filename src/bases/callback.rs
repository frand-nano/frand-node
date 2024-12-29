use std::{any::type_name_of_val, future::Future, sync::Arc};
use futures::{future::BoxFuture, FutureExt};
use super::*;

pub type EmitableFuture<M> = (NodeKey, BoxFuture<'static, M>);

#[derive(Clone)]
pub struct Callback<M: Message>(Arc<dyn Fn(PacketMessage<M>) + Send + Sync>);

#[derive(Clone)]
pub struct FutureCallback<M: Message>(Option<Arc<dyn Fn(EmitableFuture<M>) + Send + Sync>>);

impl<M: Message> std::fmt::Debug for Callback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Callback({})", type_name_of_val(&self.0))
    }
}

impl<M: Message> std::fmt::Debug for FutureCallback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FutureCallback({})", type_name_of_val(&self.0))
    }
}

impl<M: Message> Default for FutureCallback<M> {
    fn default() -> Self {
        Self(None)
    }
}

impl<M: Message> Callback<M> {
    pub fn new<F>(callback: F) -> Self 
    where F: 'static + Fn(PacketMessage<M>) + Send + Sync { 
        Self(Arc::new(callback)) 
    }

    pub fn emit<S: State>(
        &self, 
        node_key: NodeKey, 
        state: S,
    ) {
        (self.0)(
            PacketMessage {
                message: M::from_state(&node_key, 0, state)
                .unwrap_or_else(|err| panic!("{:?} {err}", node_key)),
                header: node_key.clone(),
            }                    
        );
    }
}

impl<M: Message> FutureCallback<M> {
    pub fn new<F>(callback: F) -> Self 
    where F: 'static + Fn(EmitableFuture<M>) + Send + Sync { 
        Self(Some(Arc::new(callback)))
    }

    pub fn emit<S: State, Fu>(
        &self, 
        node_key: NodeKey, 
        future: Fu,
    ) where Fu: 'static + Future<Output = S> + Send {
        if let Some(callback) = &self.0 {
            callback((node_key.clone(), async move {
                M::from_state(&node_key, 0, future.await)
                .unwrap_or_else(|err| panic!("{:?} {err}", node_key))
            }.boxed()))
        }
    }
}