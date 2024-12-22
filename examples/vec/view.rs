use eframe::egui::Ui;
use frand_node::*;
use crate::inc_button::IncButton;

pub trait VecNumberView {
    fn view(&self, ui: &mut Ui);
}

impl VecNumberView for <Vec<i32> as State>::Node {
    fn view(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let len = self.len();
    
                ui.label(format!(" len: {len} "));
    
                if ui.button(format!(" + ")).clicked() {
                    self.emit_push(0);
                }
    
                if ui.button(format!(" - ")).clicked() {
                    self.emit_pop();
                }
            });

            for item in self.items() {
                item.view(ui);
            }
        });
    }
}