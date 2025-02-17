use serde::{Deserialize, Serialize};
use frand_node::prelude::*;

mod stopwatch;
mod sum;
mod sums;

pub use self::{
    stopwatch::*,
    sum::*,
    sums::*,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Model {
    pub stopwatch: Stopwatch,
    pub sums: Sums,
}

impl System for Model {}