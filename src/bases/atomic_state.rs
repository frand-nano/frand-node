use std::fmt::Debug;
use std::sync::atomic::Ordering;
use super::*;

pub trait AtomicState<S: State>: 'static + Debug + Clone + Send + Sync {
    fn new(value: S) -> Self;
    fn load(&self, ordering: Ordering) -> S;
    fn store(&self, value: S, ordering: Ordering);    
}