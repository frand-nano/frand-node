use serde::{Deserialize, Serialize};
use frand_node::prelude::*;
use eframe::egui::*;
use crate::widget::title_frame::TitleFrame;

mod stopwatch;
mod sum;
mod sums;

pub use self::{
    stopwatch::*,
    sums::*,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Model {
    pub stopwatch: Stopwatch,
    pub sums: Sums,
}

impl System for Model {}

impl<CS: System> Widget for model::Node<'_, CS> {
    fn ui(self, ui: &mut Ui) -> Response {   
        ui.vertical(|ui| {
            ui.title_frame("Stopwatch", |ui| {
                self.stopwatch.ui(ui);
            });
  
            ui.title_frame("Sums", |ui| {
                ui.label("A 1-second delay is applied to all addition");
                self.sums.ui(ui);
            });
        }).response               
    }
}