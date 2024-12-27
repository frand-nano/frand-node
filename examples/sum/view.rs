use eframe::egui::Ui;
use crate::{clickable::IncButton, sum::*};
use frand_node::*;

pub trait SumSubView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl<M: Message> SumSubView for SumSubNode<M> {
    fn view(&self, label: &str, ui: &mut Ui) {        
        ui.horizontal(|ui| {
            ui.label(label);
            ui.inc_button(&self.a);
            ui.label(" + ");
            ui.inc_button(&self.b);
            ui.label(format!(" : {}", self.sum.v()));
        });
    }
}

pub trait SumView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl<M: Message> SumView for SumsNode<M> {
    fn view(&self, label: &str, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.sum1.view(&format!("{label}.sum1"), ui);
            self.sum2.view(&format!("{label}.sum2"), ui);
            self.total.view(&format!("{label}.total"), ui);
        });
    }
}