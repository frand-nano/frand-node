use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Timer {
    pub delta: f32,
    pub elapsed: f32,
    pub enabled: bool,
    pub reset: (),
}

impl TimerNode {
    pub fn handle(&self, message: TimerMessage) {
        use TimerMessage::*;

        match message {
            delta(d) if *self.enabled.v() => {
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