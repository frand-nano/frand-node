use std::{any::Any, fmt::Debug, future::Future};
use super::*;

#[derive(Debug, Clone)]
pub struct Emitter {
    callback: Callback,
    future_callback: FutureCallback,
    carry_callback: Callback,
}

impl Emitter {
    pub fn new(
        callback: &Callback,
        future_callback: &FutureCallback,
        carry_callback: &Callback,
    ) -> Self {
        Self { 
            callback: callback.clone(), 
            future_callback: future_callback.clone(), 
            carry_callback: carry_callback.clone(), 
        }
    }

    pub fn emit<S: State>(&self, key: NodeKey, state: S) {
        self.callback.emit(key, state)
    }

    pub fn emit_carry<S: State>(&self, key: NodeKey, state: S) {
        self.carry_callback.emit(key, state)
    }

    pub fn emit_future<S: State, Fu>(&self, key: NodeKey, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.future_callback.emit(key, future)
    }
}

pub trait Emitable: 'static + Debug + Send + Sync + Any {

}