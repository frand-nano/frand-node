use serde::{Deserialize, Serialize};
use frand_node::ext::*;
use eframe::egui::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Glow {
    pub intensity: f32,
    pub glow_sec: f32,
    value: u128,
    max: u128,
}

impl System for Glow {
    fn handle(
        node: Self::Node<'_>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    ) {
        use glow::Message::*;

        match message {
            // glow_sec 에 emit 되면 value 와 max 를 emit 하여 glow 시작
            GlowSec(sec) => {
                let max = (sec * 1000.0) as u128;
                node.value.emit(max); 
                node.max.emit(max); 
            },

            Value(value) => {          
                if 0 < value {
                    // emit 된 value 가 0 보다 클 때      
                    // (value / max) 를 intensity 에 emit
                    // (value - delta) 를 value 에 emit_carry 하여 다음 Tick 에 동작 예약
                    
                    let delta = delta.unwrap_or_default().as_millis();

                    node.intensity.emit(
                        (node.value.v() as f32) / (node.max.v() as f32)
                    );

                    node.value.emit_carry(
                        move || value.saturating_sub(delta)
                    ); 
                } else {
                    node.intensity.emit(0.0);
                }
            },

            // 그 외의 메시지를 fallback 하여 전달
            message => Self::fallback(node, message, delta),
        }        
    }
}

pub trait FillGlow {
    fn fill_glow(
        self, 
        node: &glow::Node, 
        base_color: impl Into<Color32>,
        glow_color: impl Into<Color32>,
    ) -> Self;
}

impl FillGlow for Button<'_> {
    fn fill_glow(
        self, 
        node: &glow::Node, 
        base_color: impl Into<Color32>,
        glow_color: impl Into<Color32>,
    ) -> Self {
        let base_color: Color32 = base_color.into();
        let glow_color: Color32 = glow_color.into();

        let intensity = node.intensity.v();
        let color = base_color.lerp_to_gamma(glow_color, intensity);

        self.fill(color)
    }
}

pub trait OnClickGlow {
    fn on_click_glow(self, node: &glow::Node, glow_sec: f32) -> Self;
}

impl OnClickGlow for Response {
    fn on_click_glow(self, node: &glow::Node, glow_sec: f32) -> Self {
        if self.clicked() {
            node.glow_sec.emit(glow_sec);
        }

        self
    }
}