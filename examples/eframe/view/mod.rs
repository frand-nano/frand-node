use serde::{Deserialize, Serialize};
use frand_node::*;
use eframe::egui::*;
use crate::model::{Stopwatch, Sums};

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct View {
    pub stopwatch: Proxy<Stopwatch>,
    pub sums: Proxy<Sums>,
}

impl System for View {}

impl Widget for &View {
    fn ui(self, ui: &mut Ui) -> Response {   
        ui.vertical(|ui| {
            self.stopwatch.subject().ui(ui);
            self.sums.subject().ui(ui);
        }).response        
    }
}