use std::{any::type_name_of_val, future::Future, sync::Arc};
use futures::{future::BoxFuture, FutureExt};
use super::*;

pub type EmitableFuture = (NodeKey, BoxFuture<'static, PacketMessage>);

#[derive(Clone)]
pub struct Callback(Arc<dyn Fn(PacketMessage) + Send + Sync>);

#[derive(Clone)]
pub struct FutureCallback(Arc<dyn Fn(EmitableFuture) + Send + Sync>);

impl std::fmt::Debug for Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Callback({})", type_name_of_val(&self.0))
    }
}

impl std::fmt::Debug for FutureCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FutureCallback({})", type_name_of_val(&self.0))
    }
}

impl Callback {
    pub fn new<F>(callback: F) -> Self 
    where F: 'static + Fn(PacketMessage) + Send + Sync { 
        Self(Arc::new(callback)) 
    }

    pub fn emit<S: State>(
        &self, 
        node_key: NodeKey, 
        state: S,
    ) {
        (self.0)(
            PacketMessage::new(node_key, Box::new(state))                
        )
    }
}

impl FutureCallback {
    pub fn new<F>(callback: F) -> Self 
    where F: 'static + Fn(EmitableFuture) + Send + Sync { 
        Self(Arc::new(callback))
    }

    pub fn emit<S: State, Fu>(
        &self, 
        node_key: NodeKey, 
        future: Fu,
    ) where Fu: 'static + Future<Output = S> + Send {
        (self.0)(
            (node_key.clone(), async move {
                PacketMessage::new(node_key, Box::new(future.await))
            }.boxed())                
        )
    }
}