use std::future::Future;
use super::*;

pub trait Emitter<M: Message, S: State>: Clone + Sized + Send + Sync {
    fn emit(&self, emitable: S);
    fn emit_future<Fu>(&self, future: Fu) 
    where Fu: 'static + Future<Output = S> + Send;
}