use frand_node::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Sum {
    pub a: u32,
    pub b: u32,
    pub sum: u32,
}

impl System for Sum {
    fn handle<CS: System>(
        node: Self::Node<'_, CS>, 
        message: &Self::Message, 
        delta: Option<std::time::Duration>,
    ) {
        use sum::Message::*;
        
        // Message 를 match 하여 이벤트 처리
        match message {
            // a 또는 b 에 emit 되면 a 와 b 의 합을 sum 에 emit
            A(_) | B(_) => node.sum.emit(*node.a + *node.b),

            // 그 외의 메시지를 fallback 하여 전달
            message => Self::fallback(node, message, delta),
        }     
    }
}

fn main() {
    run(1000)
}

#[test]
fn test() {
    run(10)
}

fn run(iter: u32) {
    // Sum 을 다루는 Component 를 생성
    let mut sum = Component::new(Sum::default());

    for i in 0..iter {
        // a 와 b 에 새로운 값 emit
        sum.read().node().a.emit(i * 1);
        sum.read().node().b.emit(i * 2);   
         
        // try_update 로 적용
        sum.try_update();

        // 값 확인
        assert_eq!(*sum.read().node().a, i * 1, "sum.a");
        assert_eq!(*sum.read().node().b, i * 2, "sum.b");
        assert_eq!(*sum.read().node().sum, i * 3, "sum.sum");
    }
}