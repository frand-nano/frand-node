## eframe 을 이용한 GUI 예제 모음입니다.

### Stopwatch : emit_carry 를 활용하여 여러 Tick 에 걸쳐 일어나는 작업을 처리하는 예제입니다.
[Stopwatch](https://github.com/frand-nano/frand-node/blob/main/examples/eframe/model/stopwatch.rs)

```rust
impl System for Stopwatch {
    fn handle<CS: System>(
        node: Self::Node<'_, CS>, 
        message: &Self::Message, 
        delta: Option<std::time::Duration>,
    ) {        
        use stopwatch::Message::*;

        match message {
            // elapsed 가 emit 되고 enabled 가 true 일때
            // 이전 Tick 으로부터의 delta 를 elapsed 에 더하여
            // elapsed.emit_carry() 를 호출하여 다음 Tick 에 동작 예약
            Elapsed(elapsed) if *node.enabled => {
                let delta = delta.unwrap_or_default().as_secs_f32();
                node.elapsed.emit_carry(elapsed + delta);
            },

            // enabled 에 true 가 emit 되었을 때
            // elapsed 를 emit 하여 elapsed 를 재시동
            Enabled(enabled) if *enabled => {
                node.elapsed.emit(*node.elapsed);
            },

            // reset 이 emit 되었을 때 
            // enabled 와 elapsed 를 emit 하여 초기화 및 정지
            Reset(_) => {
                node.enabled.emit(false);
                node.elapsed.emit_carry(0.0);
            },

            // 그 외의 메시지를 fallback 하여 전달
            message => Self::fallback(node, message, delta),
        }       
    }
}
```

### Sums : VecNode 와 emit_future 를 활용하여 Vec 과 async 를 조합한 비동기 더하기 예제입니다.
[Sums](https://github.com/frand-nano/frand-node/blob/main/examples/eframe/model/sums.rs)

```rust
impl System for Sums {
    fn handle<CS: System>(
        node: Self::Node<'_, CS>, 
        message: &Self::Message, 
        delta: Option<std::time::Duration>,
    ) {        
        use sums::Message::*;
        use sum::Message::*;
        use vec::Message::*;

        match message {
            // values 에 push 또는 pop 이 emit 되면 sums 에 push 또는 pop 을 emit 하여 길이 동기화
            Values(Push(item)) => node.sums.emit_push(item.sum),
            Values(Pop) => node.sums.emit_pop(),

            // values 의 index 번째 item 에 sum 이 emit 되었을 때
            // sums 의 index 번째 item 에 sum 을 emit
            Values(Item(index, Sum(sum))) => {
                node.sums.item(*index).emit(*sum)
            },            

            // sums 에 emit 되었을 때
            // sums 의 모든 값들을 Box에 모아 1초뒤에 그 합을 emit
            Sums(_) => {
                let values: Box<_> = node.sums.items().map(|n| *n).collect();
                node.total.emit_future(async move {
                    sleep(Duration::from_millis(1000)).await;
                    values.iter().sum()
                })
            },       

            // 그 외의 메시지를 fallback 하여 전달
            // values: Vec<Sum> 
            // Sum Node 는 a, b, sum 을 가지며 a 또는 b 에 emit 되면 sum 에 그 합을 emit
            message => Self::fallback(node, message, delta),
        }             
    }
}
```