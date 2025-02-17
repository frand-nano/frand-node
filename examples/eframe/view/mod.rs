use serde::{Deserialize, Serialize};
use frand_node::prelude::*;
use eframe::egui::*;

mod title_frame;
mod glow;
mod stopwatch;
mod sum;
mod sums;

pub use self::{
    title_frame::*,
    glow::*,
    stopwatch::*,
    sums::*,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct View {
    pub stopwatch: StopwatchView,
    pub sums: SumsView,
}

impl System for View {}

impl Widget for view::Node<'_> {
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