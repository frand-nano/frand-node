use anyhow::Result;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
struct Sums {
    sum1: SumSub,
    sum2: SumSub,
    total: SumSub,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
struct SumSub {
    a: i32,
    b: i32,
    sum: i32,
}

impl SumSubStateNode<'_> {
    // SumSub 의 a 와 b 의 합을 sum 에 emit()
    fn emit_sum(&self) {
        self.sum.emit(*self.a + *self.b)
    }
}

impl Sums {
    fn update(node: &SumsStateNode<'_>, message: SumsMessage) {
        use SumsMessage::*;
        use SumSubMessage::*;

        // Message 를 match 하여 이벤트 처리
        match message {
            // sum1 의 a 또는 b 가 emit 되면 sum1.sum 에 sum1.a + sum1.b 를 emit
            // sum1 의 sum 이 emit 되면 total.a 에 sum1.sum 을 emit
            sum1(a(_) | b(_)) => node.sum1.emit_sum(),
            sum1(sum(s)) => node.total.a.emit(s),

            sum2(a(_) | b(_)) => node.sum2.emit_sum(),
            sum2(sum(s)) => node.total.b.emit(s),

            total(a(_) | b(_)) => node.total.emit_sum(),

            _ => (),
        }
    }
}

#[test]
fn sum() -> Result<()> { main() }

fn main() -> Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();
    
    // Sums 를 다루는 Processor 를 생성
    let mut processor = Processor::<Sums>::new(
        // emit() 으로 발생한 이벤트 콜백
        |result| if let Err(err) = result { log::info!("{err}") }, 
        // Message 를 처리하기 위한 Sums::update 함수 연결
        Sums::update,
    );

    processor.node().sum1.a.emit(1);
    processor.node().sum1.b.emit(2);
    processor.node().sum2.a.emit(3);
    processor.node().sum2.b.emit(4);

    processor.process()?;

    assert_eq!(processor.sum1.a, 1, "sum1.a");
    assert_eq!(processor.sum1.b, 2, "sum1.b");
    assert_eq!(processor.sum1.sum, 1 + 2, "sum1.sum");

    assert_eq!(processor.sum2.a, 3, "sum2.a");
    assert_eq!(processor.sum2.b, 4, "sum2.b");
    assert_eq!(processor.sum2.sum, 3 + 4, "sum2.sum");

    assert_eq!(processor.total.a, 1 + 2, "total.a");
    assert_eq!(processor.total.b, 3 + 4, "total.b");
    assert_eq!(processor.total.sum, 1 + 2 + 3 + 4, "total.sum");

    Ok(())
}
