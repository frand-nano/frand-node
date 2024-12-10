use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Timer {
    pub delta: f32,
    pub elapsed: f32,
    pub enabled: bool,
    pub reset: (),
}

impl Timer {
    pub fn update(node: &TimerStateNode, message: TimerMessage) {
        use TimerMessage::*;

        match message {
            delta(d) if *node.enabled => {
                node.elapsed.emit(*node.elapsed + d);
            },
            reset(_) => {
                node.elapsed.emit(0f32);
                node.enabled.emit(false);
            },
            _ => (),
        }
    }
}