use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Stopwatch {
    pub delta: f32,
    pub elapsed: f32,
    pub enabled: bool,
    pub reset: (),
}

impl<M: Message> StopwatchNode<M> {
    pub fn handle(&self, message: StopwatchMessage) {
        use StopwatchMessage::*;

        match message {
            delta(d) if self.enabled.v() => {
                self.elapsed.emit(*self.elapsed.v() + d);
            },
            reset(_) => {
                self.elapsed.emit(0f32);
                self.enabled.emit(false);
            },
            _ => (),
        }
    }
}