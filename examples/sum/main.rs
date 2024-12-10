use anyhow::Result;
use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
struct Sum {
    sum1: SumSub,
    sum2: SumSub,
    sum3: SumSub,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
struct SumSub {
    a: i32,
    b: i32,
    sum: i32,
}

impl SumSubStateNode<'_> {
    pub fn emit_sum(&self) -> Result<()> {
        self.sum.emit(*self.a + *self.b)
    }
}

#[test]
fn sum() -> Result<()> { main() }

fn main() -> Result<()> {
    let mut processor = Processor::<Sum>::new(
        || {},
        |node, message| {
            use SumMessage::*;
            use SumSubMessage::*;

            Ok(match message {
                sum1(a(_) | b(_)) => node.sum1.emit_sum()?,
                sum1(sum(s)) => node.sum3.a.emit(s)?,

                sum2(a(_) | b(_)) => node.sum2.emit_sum()?,
                sum2(sum(s)) => node.sum3.b.emit(s)?,

                sum3(a(_) | b(_)) => node.sum3.emit_sum()?,

                _ => (),
            })
        }
    );

    processor.node().sum1.a.emit(1)?;
    processor.node().sum1.b.emit(2)?;
    processor.node().sum2.a.emit(3)?;
    processor.node().sum2.b.emit(4)?;
    processor.process()?;

    assert_eq!(processor.state().sum1.a, 1, "sum1.a");
    assert_eq!(processor.state().sum1.b, 2, "sum1.b");
    assert_eq!(processor.state().sum1.sum, 1 + 2, "sum1.sum");

    assert_eq!(processor.state().sum2.a, 3, "sum2.a");
    assert_eq!(processor.state().sum2.b, 4, "sum2.b");
    assert_eq!(processor.state().sum2.sum, 3 + 4, "sum2.sum");

    assert_eq!(processor.state().sum3.a, 1 + 2, "sum3.a");
    assert_eq!(processor.state().sum3.b, 3 + 4, "sum3.b");
    assert_eq!(processor.state().sum3.sum, 1 + 2 + 3 + 4, "sum3.sum");

    Ok(())
}
