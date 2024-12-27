use eframe::egui::Ui;
use crate::stopwatch::*;
use frand_node::*;

pub trait StopwatchView {
    fn view(&self, ui: &mut Ui);
}

impl<M: Message> StopwatchView for StopwatchNode<M> {
    fn view(&self, ui: &mut Ui) {        
        ui.label(format!("elapsed : {:.1}", *self.elapsed.v()));

        let start_stop_text = if self.enabled.v() { "stop" } else { "start" };

        if ui.button(start_stop_text).clicked() {
            self.enabled.emit(!self.enabled.v());
        }

        if ui.button("reset").clicked() {
            self.reset.emit(());
        }
    }
}
