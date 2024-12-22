use std::sync::atomic::Ordering;
use super::*;

pub trait AtomicState: State + Copy {
    type Atomic: Atomic<Self>;
}

pub trait Atomic<S: AtomicState>: Clone + Send + Sync {
    fn new(value: S) -> Self;
    fn load(&self, ordering: Ordering) -> S;
    fn store(&self, value: S, ordering: Ordering);    
}