use eframe::egui::Ui;
use extends::OptionNode;
use frand_node::*;
use crate::inc_on_click::IncButton;

pub trait OptionNumberView {
    fn view(&self, ui: &mut Ui);
}

impl OptionNumberView for OptionNode<i32> {
    fn view(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let is_some = self.item().is_some();

            let label = if is_some { " Some " } else { " None " };

            if ui.button(label).clicked() {
                self.is_some.emit(!is_some);
            }
            
            if let Some(number) = self.item() {          
                ui.inc_button(&number);
            }
        });
    }
}