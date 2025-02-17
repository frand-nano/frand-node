use std::{fmt::Debug, future::Future};
use crate::ext::*;

pub trait Emitter<S: State>: Debug + Clone + Send + Sync {
    fn callback(&self) -> &Callback<S>;

    fn new(
        callback: Callback<S>,
    ) -> Self;
    
    fn emit(
        &self, 
        callback_mode: &CallbackMode, 
        transient: &Transient, 
        state: S,
    ) {
        self.callback().emit(
            callback_mode, 
            transient, 
            state.into_message()
        );
    }

    fn emit_carry<F>(
        &self, 
        callback_mode: &CallbackMode, 
        transient: &Transient, 
        lookup: F,
    ) where F: Fn() -> S::Message + 'static + Send + Sync {
        self.callback().emit_carry(
            callback_mode, 
            transient, 
            lookup,
        );
    }

    fn emit_future<F>(
        &self, 
        callback_mode: &CallbackMode, 
        transient: &Transient, 
        future: F,
    ) 
    where F: Future<Output = S::Message> + 'static + Send + Sync {
        self.callback().emit_future(
            callback_mode, 
            transient, 
            future,
        );
    }
}