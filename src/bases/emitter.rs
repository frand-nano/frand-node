use std::{fmt::Debug, future::Future};
use crate::ext::*;

pub trait Emitter<S: State>: Debug + Clone + Send + Sync {
    fn callback(&self) -> &Callback<S::Message>;

    fn new(
        callback: Callback<S::Message>,
    ) -> Self;
    
    fn emit(&self, transient: &Transient, state: S) {
        self.callback().emit(transient, state.into_message());
    }

    fn emit_carry(&self, transient: &Transient, state: S) {
        self.callback().emit_carry(transient, state.into_message());
    }

    fn emit_future<F>(&self, transient: &Transient, future: F) 
    where F: Future<Output = S::Message> + 'static + Send + Sync {
        self.callback().emit_future(transient, future);
    }
}