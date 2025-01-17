use serde::{Deserialize, Serialize};
use eframe::egui::*;
use frand_node::*;
use crate::widget::title_frame::TitleFrame;

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stopwatch {
    pub elapsed: f32,
    pub enabled: bool,
    pub reset: (),
}

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

impl Widget for &Stopwatch {
    fn ui(self, ui: &mut Ui) -> Response {       
        ui.title_frame("Stopwatch", |ui| {
            ui.label(format!("elapsed : {:.1}", *self.elapsed.v()));

            let start_stop_text = if self.enabled.v() { 
                "stop" 
            } else { 
                "start" 
            };
    
            if ui.button(start_stop_text).clicked() {
                self.enabled.emit(!self.enabled.v());
            }
    
            if ui.button("reset").clicked() {
                self.reset.emit(());
            }
        }).response        
    }
}