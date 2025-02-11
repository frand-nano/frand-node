# FrandNode

* Rust 의 Macro 와 Trait 을 활용하여 MultiThread 및 MultiClient 환경에서 Async Event Message를 처리하는 도구를 제공합니다.


## 구성

* **State**: 구조체 또는 일부 primitives 입니다.
* **Consensus**: 하나의 **State** 를 소유하고 관련 메시지의 입출력을 위한 도구를 제공합니다. Clone, Send, Sync 를 구현합니다.
* **Node**: 계층 구조를 구성하는 노드입니다. 값을 읽거나 값 변경을 위한 메시지를 보낼 수 있습니다.
* **NodeAlt**: **Consensus** 에 대한 읽기 컨텍스트입니다. Vec 등의 컬렉션 지원을 위한 정보를 보관합니다.


* **Component**: 하나의 **Consensus** 를 소유하고 메시지의 연쇄 적용과 비동기 처리를 담당합니다.


* **Packet**: 메시지의 바이트 직렬화를 매개합니다.


## 예시 

* [examples/eframe](https://github.com/frand-nano/frand-node/blob/main/examples/eframe)
* [examples/sum](https://github.com/frand-nano/frand-node/blob/main/examples/sum)

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/frand-nano/frand-node/blob/main/LICENSE