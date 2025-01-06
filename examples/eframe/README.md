## eframe 을 이용한 GUI 예제 모음입니다.

### Sums : VecNode 와 emit_future 를 활용하여 Vec 과 async 를 조합한 비동기 더하기 예제입니다.
[Sums](https://github.com/frand-nano/frand-node/blob/main/examples/eframe/model/sums.rs)

```rust
impl System for Sums {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        use SumsMessage::*;
        use SumMessage::*;
        use VecMessage::*;

        match message {
            // values 에 push 또는 pop 이 emit 되면 sums 에 push 또는 pop 을 emit 하여 길이 동기화
            Values(Push(_)) => self.sums.emit_push(Default::default()),
            Values(Pop(_)) => self.sums.emit_pop(),

            // sums 에 push 또는 pop 이 emit 되면 values 에 push 또는 pop 을 emit 하여 길이 동기화
            Sums(Push(_)) => self.values.emit_push(Default::default()),
            Sums(Pop(_)) => self.values.emit_pop(),

            // values 의 index 번째 item 에 sum 이 emit 되었을 때
            // sums 의 index 번째 item 에 sum 을 emit
            Values(Item((index, Sum(sum)))) => self.sums.items()[index as usize].emit(sum),            

            // sums 에 emit 되었을 때
            // sums 의 모든 값들을 Box에 모아 1초뒤에 그 합을 emit
            Sums(_) => {
                let values: Box<_> = self.sums.items().map(|n| n.v()).collect();
                self.total.emit_future(async move {
                    sleep(Duration::from_millis(1000)).await;
                    values.iter().sum()
                })
            },       

            // 그 외의 메시지를 fallback 하여 전달
            // values: Vec<Sum> 
            // Sum Node 는 a, b, sum 을 가지며 a 또는 b 에 emit 되면 sum 에 그 합을 emit
            message => self.fallback(message, delta)
        }        
    }
}
```

### Stopwatch : emit_carry 를 활용하여 여러 Tick 에 걸쳐 일어나는 작업을 처리하는 예제입니다.
[Stopwatch](https://github.com/frand-nano/frand-node/blob/main/examples/eframe/model/stopwatch.rs)

```rust
impl System for Stopwatch {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        use StopwatchMessage::*;

        match message {
            // elapsed 가 emit 되고 enabled 가 true 일때
            // 이전 Tick 으로부터의 delta 를 elapsed 에 더하여
            // elapsed.emit_carry() 를 호출하여 다음 Tick 에 동작 예약
            Elapsed(elapsed) if self.enabled.v() => {
                let delta = delta.unwrap_or_default();
                self.elapsed.emit_carry(elapsed + delta);
            },

            // enabled 에 true 가 emit 되었을 때
            // elapsed 를 emit 하여 elapsed 를 재시동
            Enabled(enabled) if enabled => {
                self.elapsed.emit(*self.elapsed.v());
            },

            // reset 이 emit 되었을 때 
            // enabled 와 elapsed 를 emit 하여 초기화 및 정지
            Reset(_) => {
                self.enabled.emit(false);
                self.elapsed.emit_carry(0.0);
            },

            // 그 외의 메시지를 fallback 하여 전달
            message => self.fallback(message, delta)
        }
    }
}
```