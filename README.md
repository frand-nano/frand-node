# FrandNode

* Rust 의 Macro 와 Trait 을 활용하여 Async Event Message 를 다루는 도구들을 제공합니다.
* **Node** 를 clone() 하여 MultiThread 환경이나 ViewModel 에서 Callback 을 활용하거나
* **Packet** 으로 메시지 처리 파이프라인을 만들거나 Server와 Client 의 상태를 동기화하는 작업 등에 활용할 수 있습니다.


## 구조

* **State**: 구조체 또는 일부 primitives 입니다.
* **Message**: 데이터 변경의 타겟과 값을 가지는 enum 계층 구조를 제공합니다.
* **Node**: 데이터 변경에 대한 Event 를 Callback 할 수 있는 계층 구조를 제공합니다.
* **Packet**: `[u8]` 로 Serialize, Deserialize 될 수 있는 구조체입니다. **Node** 로부터 생성되며 **Message** 로 변환되거나 **State** 에 값을 적용하는 용도로 사용할 수 있습니다.

* **Processor**: callback을 지정하여 input, output channel 과 **Node** 를 연계합니다. **Message** 를 match 하여 Event 를 연쇄 적용합니다. 하나의 **Packet** 으로부터 하나 이상의 **Packet** 을 생성하고 **State** 에 적용하는 방식으로 동작합니다. emit 된 future 들은 비동기적으로 동시 처리됩니다.

## 예시 

* [examples/eframe](https://github.com/frand-nano/frand-node/blob/main/examples/eframe)
* [examples/profile_sum](https://github.com/frand-nano/frand-node/blob/main/examples/profile_sum)

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/frand-nano/frand-node/blob/main/LICENSE