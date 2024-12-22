#[test]
fn sum() -> anyhow::Result<()> {
    use log::LevelFilter;
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use frand_node::*;
    use crate::Sums;
    
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::warn!("{err}"));
    
    // Sums 를 다루는 Processor 를 생성
    let mut processor = Processor::<Sums>::new(
        // emit() 으로 발생한 이벤트 콜백
        |result| if let Err(err) = result { log::error!("{err}") }, 
        // Message 처리
        |node, message| node.handle(message),
    );

    for _ in 0..1 {
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