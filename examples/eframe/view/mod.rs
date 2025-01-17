use bases::Index;
use serde::{Deserialize, Serialize};
use frand_node::*;
use eframe::egui::*;
use crate::{model::{Stopwatch, Sum, Sums}, widget::title_frame::TitleFrame};

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct View {
    pub stopwatch: Proxy<Stopwatch>,
    pub sums: Proxy<Sums>,
    pub selected_sum: Proxy<Sums, Index, Sum>,
}

impl System for View {}

impl Widget for &View {
    fn ui(self, ui: &mut Ui) -> Response {   
        ui.vertical(|ui| {
            self.stopwatch.subject().unwrap().ui(ui);
            self.sums.subject().unwrap().ui(ui);

            ui.title_frame("Selected Sum", |ui| {
                ui.horizontal(|ui| {
                    let locate = self.selected_sum.locate();
                    let index = locate.clone_state();
        
                    if ui.button(format!(" - ")).clicked() {
                        locate.emit(index.saturating_sub(1))
                    }
        
                    ui.label(format!(" index: {:?} ", index));
        
                    if ui.button(format!(" + ")).clicked() {                    
                        locate.emit(index.saturating_add(1))
                    }
                });                

                match self.selected_sum.subject() {
                    Some(sum) => sum.ui(ui),
                    None => ui.label("None"),
                }
            });
        }).response        
    }
}