use eframe::egui::Ui;
use extends::VecNode;
use frand_node::*;
use crate::clickable::IncButton;

pub trait VecNumberView {
    fn view(&self, ui: &mut Ui);
}

impl<M: Message> VecNumberView for VecNode<M, i32> {
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
                ui.inc_button(&item); 
            }
        });
    }
}