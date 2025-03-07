use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Stopwatch {
    pub elapsed: f32,
    pub enabled: bool,
    pub reset: (),
}

impl System for Stopwatch {
    fn handle(
        node: Self::Node<'_>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    ) {        
        use stopwatch::Message::*;

        match message {
            // elapsed 가 emit 되고 enabled 가 true 일때
            // 이전 Tick 으로부터의 delta 를 elapsed 에 더하여
            // elapsed.emit_carry() 를 호출하여 다음 Tick 에 동작 예약
            Elapsed(elapsed) if node.enabled.v() => {
                let delta = delta.unwrap_or_default().as_secs_f32();

                node.elapsed.emit_carry(
                    move || elapsed + delta
                );
            },

            // enabled 에 true 가 emit 되었을 때
            // elapsed 를 emit 하여 elapsed 를 재시동
            Enabled(enabled) if enabled => {
                node.elapsed.emit(node.elapsed.v());
            },

            // reset 이 emit 되었을 때 
            // enabled 와 elapsed 를 emit 하여 초기화 및 정지
            Reset(_) => {
                node.enabled.emit(false);
                node.elapsed.emit_carry(|| 0.0);
            },

            // 그 외의 메시지를 fallback 하여 전달
            message => Self::fallback(node, message, delta),
        }       
    }
}