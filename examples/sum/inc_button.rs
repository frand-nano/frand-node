use std::{fmt::Display, ops::Add};
use eframe::egui::Ui;
use frand_node::*;

pub trait IncButton<S: State>: Node<S> 
where S: Display + Add<i32, Output = S> {
    fn view(&self, ui: &mut Ui) {
        let value = self.clone_state();

        if ui.button(format!(" {value} ")).clicked() {
            self.emit(value + 1);
        }
    }
}

impl IncButton<i32> for <i32 as State>::Node {}