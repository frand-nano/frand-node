use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Sums {
    pub sum1: SumSub,
    pub sum2: SumSub,
    pub total: SumSub,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct SumSub {
    pub a: i32,
    pub b: i32,
    pub sum: i32,
}

impl SumSubStateNode<'_> {
    pub fn emit_sum(&self) {
        self.sum.emit(*self.a + *self.b)
    }
}

impl Sums {
    pub fn update(node: &SumsStateNode<'_>, message: SumsMessage) {
        use SumsMessage::*;
        use SumSubMessage::*;

        match message {
            sum1(a(_) | b(_)) => node.sum1.emit_sum(),
            sum1(sum(s)) => node.total.a.emit(s),

            sum2(a(_) | b(_)) => node.sum2.emit_sum(),
            sum2(sum(s)) => node.total.b.emit(s),

            total(a(_) | b(_)) => node.total.emit_sum(),

            _ => (),
        }
    }
}