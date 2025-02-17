use serde::{Deserialize, Serialize};
use eframe::egui::*;
use frand_node::{proxy::Proxy, *};
use crate::model::Stopwatch;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct StopwatchView {
    pub model: Proxy<Stopwatch>,    
}

impl System for StopwatchView {}

impl Widget for stopwatch_view::Node<'_> {
    fn ui(self, ui: &mut Ui) -> Response {      
        ui.horizontal(|ui| {
            if let Some(model) = self.model.subject() { 
                ui.label(format!("elapsed : {:.1}", model.elapsed.v()));
    
                let start_stop_text = if model.enabled.v() { 
                    "stop" 
                } else { 
                    "start" 
                };
        
                if ui.button(start_stop_text).clicked() {
                    model.enabled.emit(!model.enabled.v());
                }
        
                if ui.button("reset").clicked() {
                    model.reset.emit(());
                }   
            } else {
                ui.label("The model is not applied.");
            }       
        }).response     
    }
}