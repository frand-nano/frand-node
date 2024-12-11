use std::{fmt::Display, ops::Add};
use eframe::egui::Ui;
use frand_node::*;
use crate::sum::*;

pub trait IncButton<'a, S: State>: Node<'a, S> 
where S: Display + Add<i32, Output = S> {
    fn view(&self, ui: &mut Ui) {
        let value = self.clone_state();

        if ui.button(format!(" {value} ")).clicked() {
            self.emit(value + 1);
        }
    }
}

impl<'a> IncButton<'a, i32> for <i32 as State>::Node<'a> {}

pub trait SumSubView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl SumSubView for SumSubNode<'_> {
    fn view(&self, label: &str, ui: &mut Ui) {        
        ui.horizontal(|ui| {
            ui.label(label);
            self.a.view(ui);
            ui.label(" + ");
            self.b.view(ui);
            ui.label(format!(" : {}", *self.sum));
        });
    }
}

pub trait SumView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl SumView for SumsNode<'_> {
    fn view(&self, label: &str, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.sum1.view(&format!("{label}.sum1"), ui);
            self.sum2.view(&format!("{label}.sum2"), ui);
            self.total.view(&format!("{label}.total"), ui);
        });
    }
}