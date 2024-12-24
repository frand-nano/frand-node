use extends::Processor;
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

fn main() -> anyhow::Result<()> {
    // Sums 를 다루는 Processor 를 생성
    let mut processor = Processor::<Sums>::new(
        // emit() 으로 발생한 이벤트 콜백
        |result| if let Err(err) = result { log::error!("{err}") }, 
        // Message 처리
        |node, message| node.handle(message),
    );

    for _ in 0..10000 {
        processor.sum1.a.emit(1);
        processor.sum1.b.emit(2);
        processor.sum2.a.emit(3);
        processor.sum2.b.emit(4);
    
        processor.process()?;
    }

    assert_eq!(processor.sum1.a.v(), 1, "sum1.a");
    assert_eq!(processor.sum1.b.v(), 2, "sum1.b");
    assert_eq!(processor.sum1.sum.v(), 1 + 2, "sum1.sum");

    assert_eq!(processor.sum2.a.v(), 3, "sum2.a");
    assert_eq!(processor.sum2.b.v(), 4, "sum2.b");
    assert_eq!(processor.sum2.sum.v(), 3 + 4, "sum2.sum");

    assert_eq!(processor.total.a.v(), 1 + 2, "total.a");
    assert_eq!(processor.total.b.v(), 3 + 4, "total.b");
    assert_eq!(processor.total.sum.v(), 1 + 2 + 3 + 4, "total.sum");

    Ok(())
}