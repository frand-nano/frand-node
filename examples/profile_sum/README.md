## 4개의 i32 값을 더해서 2개의 합과 하나의 total 값을 구하는 간단한 예제입니다.

```rust
#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Sums {
    pub sum1: Sum,
    pub sum2: Sum,
    pub total: Sum,
}

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Sum {
    pub a: i32,
    pub b: i32,
    pub sum: i32,
}
```

* **Message** 처리 함수 작성
```rust
impl System for Sums {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        use SumsMessage::*;        
        use SumMessage::*;

        // Message 를 match 하여 이벤트 처리
        match message {
            // sum1.sum 에 emit 되면 total.a 에 emit
            Sum1(Sum(s)) => self.total.a.emit(s),

            // sum2.sum 에 emit 되면 total.b 에 emit
            Sum2(Sum(s)) => self.total.b.emit(s),

            // 그 외의 메시지를 fallback 하여 전달
            message => self.fallback(message, delta)
        }        
    }
}

impl System for Sum {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        use SumMessage::*;

        // Message 를 match 하여 이벤트 처리
        match message {
            // a 또는 b 에 emit 되면 a 와 b 의 합을 sum 에 emit
            A(_) | B(_) => self.sum.emit(self.a.v() + self.b.v()),
            
            // 그 외의 메시지를 fallback 하여 전달
            message => self.fallback(message, delta)
        }        
    }
}
```

* **Processor** 생성
```rust
// Sums 를 다루는 Processor 를 생성
let mut processor = Processor::<Sums>::new(
    // emit() 으로 발생한 이벤트 콜백
    |result| if let Err(err) = result { log::error!("{err}") }, 
    // Message 처리
    |node, message| node.handle(message),
);
```

* **Processor** 의 **Node** 에 새로운 값을 emit
```rust
processor.sum1.a.emit(1);
processor.sum1.b.emit(2);
processor.sum2.a.emit(3);
processor.sum2.b.emit(4);
```

* process() 로 적용 후 테스트
```rust
processor.process()?;

assert_eq!(processor.sum1.a.v(), 1, "sum1.a");
assert_eq!(processor.sum1.b.v(), 2, "sum1.b");
assert_eq!(processor.sum1.sum.v(), 1 + 2, "sum1.sum");

assert_eq!(processor.sum2.a.v(), 3, "sum2.a");
assert_eq!(processor.sum2.b.v(), 4, "sum2.b");
assert_eq!(processor.sum2.sum.v(), 3 + 4, "sum2.sum");

assert_eq!(processor.total.a.v(), 1 + 2, "total.a");
assert_eq!(processor.total.b.v(), 3 + 4, "total.b");
assert_eq!(processor.total.sum.v(), 1 + 2 + 3 + 4, "total.sum");
```
