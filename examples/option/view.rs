use eframe::egui::Ui;
use extends::OptionNode;
use frand_node::*;
use crate::clickable::IncButton;

pub trait OptionNumberView {
    fn view(&self, ui: &mut Ui);
}

impl<M: Message> OptionNumberView for OptionNode<M, i32> {
    fn view(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let is_some = self.item().is_some();

            let label = if is_some { " Some " } else { " None " };

            if ui.button(label).clicked() {
                self.is_some.emit(!is_some);
            }
            
            if let Some(number) = self.item() {          
                ui.inc_button(number);
            }
        });
    }
}