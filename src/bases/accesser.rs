use std::fmt::Debug;
use crate::ext::*;

pub trait Accesser<S: State>: Debug + Clone + Send + Sync {
    fn lookup(&self) -> &Lookup<S>;

    fn new<CS: System>(builder: LookupBuilder<CS, S>) -> Self;
}
