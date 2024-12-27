# FrandNode

* Rust 의 Derive Macro 와 Trait 을 활용하여 Async Event Message 를 다루는 도구들을 제공합니다.
* **Node** 를 clone() 하여 MultiThread 환경이나 ViewModel 에서 Callback 을 활용하거나
* **Packet** 으로 메시지 처리 파이프라인을 만들거나 Server와 Client 의 상태를 동기화하는 작업 등에 활용할 수 있습니다.


## 구조

* **State**: 구조체 또는 일부 primitives 입니다.
* **Message**: 데이터 변경의 타겟과 값을 가지는 enum 계층 구조를 제공합니다.
* **Node**: 데이터 변경에 대한 Event 를 Callback 할 수 있는 계층 구조를 제공합니다.
* **Packet**: `[u8]` 로 Serialize, Deserialize 될 수 있는 구조체입니다. **Node** 로부터 생성되며 **Message** 로 변환되거나 **State** 에 값을 적용하는 용도로 사용할 수 있습니다.

* **Consensus**: **Node** 의 생성과 **State**, **Packet** 의 적용을 담당합니다. 
* **Container**: callback을 지정하여 input, output channel 과 **Node** 를 연계합니다.
* **Processor**: **Container** 에 더하여 **Message** 를 match 하여 Event 를 연쇄 적용합니다. 하나의 **Packet** 으로부터 하나 이상의 **Packet** 을 생성하고 **State** 에 적용하는 방식으로 동작합니다.
* **AsyncProcessor**: **Processor** 에 더하여 **Node** 에 emit 된 future 들을 비동기적으로 동시 처리합니다.

## 예시 

* [examples/sum](https://github.com/frand-nano/frand-node/blob/main/examples/sum)
* [examples/stopwatch](https://github.com/frand-nano/frand-node/blob/main/examples/stopwatch)
* [examples/async_sum](https://github.com/frand-nano/frand-node/blob/main/examples/async_sum)
* [examples/option](https://github.com/frand-nano/frand-node/blob/main/examples/option)
* [examples/vec](https://github.com/frand-nano/frand-node/blob/main/examples/vec)


아래는 [examples/sum](https://github.com/frand-nano/frand-node/blob/main/examples/sum) 예시입니다.

* `#[derive(Node)]`
```rust
#[derive(Node)]
pub struct Sums {
    pub sum1: SumSub,
    pub sum2: SumSub,
    pub total: SumSub,
}

#[derive(Node)]
pub struct SumSub {
    pub a: i32,
    pub b: i32,
    pub sum: i32,
}
```

* **Message** 처리 함수 작성
```rust
impl<M: Message> SumsNode<M> {
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
```

```rust
impl<M: Message> SumSubNode<M> {
    // SumSub 의 a 와 b 의 합을 sum 에 emit()
    fn emit_sum(&self) {
        self.sum.emit(self.a.v() + self.b.v())
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


## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/frand-nano/frand-node/blob/main/LICENSE