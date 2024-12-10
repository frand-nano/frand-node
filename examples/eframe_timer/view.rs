use eframe::egui::Ui;
use crate::timer::*;
use frand_node::*;

pub trait TimerView {
    fn view(&self, ui: &mut Ui);
}

impl TimerView for TimerStateNode<'_> {
    fn view(&self, ui: &mut Ui) {        
        ui.label(format!("elapsed : {:.1}", *self.elapsed));

        let start_stop_text = if *self.enabled { "stop" } else { "start" };

        if ui.button(start_stop_text).clicked() {
            self.enabled.emit(!*self.enabled);
        }

        if ui.button("reset").clicked() {
            self.reset.emit(());
        }
    }
}
