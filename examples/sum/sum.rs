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

impl SumsNode {
    pub fn handle(&self, message: SumsMessage) {
        use SumsMessage::*;
        use SumSubMessage::*;

        // Message 를 match 하여 이벤트 처리
        match message {
            // sum1 의 a 또는 b 가 emit 되면 sum1.sum 에 sum1.a + sum1.b 를 emit
            // sum1 의 sum 이 emit 되면 total.a 에 sum1.sum 을 emit
            sum1(a(_) | b(_)) => self.sum1.emit_sum(),
            sum1(sum(s)) => self.total.a.emit(s),

            sum2(a(_) | b(_)) => self.sum2.emit_sum(),
            sum2(sum(s)) => self.total.b.emit(s),

            total(a(_) | b(_)) => self.total.emit_sum(),

            _ => (),
        }
    }
}

impl SumSubNode {
    // SumSub 의 a 와 b 의 합을 sum 에 emit()
    fn emit_sum(&self) {
        self.sum.emit(self.a.v() + self.b.v())
    }
}
