use serde::{Deserialize, Serialize};
use frand_node::*;

mod stopwatch;
mod sum;
mod sums;

pub use self::{
    stopwatch::*,
    sum::*,
    sums::*,
};

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Model {
    pub stopwatch: Stopwatch,
    pub sums: Sums,
}

impl System for Model {}

