use std::{any::Any, fmt::Debug, future::Future};
use super::*;

#[derive(Debug, Clone)]
pub struct Emitter {
    callback: Callback,
    carry_callback: Callback,
    future_callback: FutureCallback,
}

impl Emitter {
    pub fn new(
        callback: Callback,
        carry_callback: Callback,
        future_callback: FutureCallback,
    ) -> Self {
        Self { 
            callback, 
            carry_callback, 
            future_callback, 
        }
    }

    pub fn emit<S: State>(&self, key: Key, state: S) {
        self.callback.emit(key, state)
    }

    pub fn emit_carry<S: State>(&self, key: Key, state: S) {
        self.carry_callback.emit(key, state)
    }

    pub fn emit_future<S: State, Fu>(&self, key: Key, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send {
        self.future_callback.emit(key, future)
    }
}

pub trait Emitable: 'static + Debug + Send + Sync + Any {

}