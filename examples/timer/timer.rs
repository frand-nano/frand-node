use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Timer {
    pub delta: f32,
    pub elapsed: f32,
    pub enabled: bool,
    pub reset: (),
}

impl TimerStateNode<'_> {
    pub fn handle(&self, message: TimerMessage) {
        use TimerMessage::*;

        match message {
            delta(d) if *self.enabled => {
                self.elapsed.emit(*self.elapsed + d);
            },
            reset(_) => {
                self.elapsed.emit(0f32);
                self.enabled.emit(false);
            },
            _ => (),
        }
    }
}