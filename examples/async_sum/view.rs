use eframe::egui::Ui;
use crate::{inc_button::IncButton, sum::*};

pub trait SumSubView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl SumSubView for SumSubNode {
    fn view(&self, label: &str, ui: &mut Ui) {        
        ui.horizontal(|ui| {
            ui.label(label);
            self.a.view(ui);
            ui.label(" + ");
            self.b.view(ui);
            ui.label(format!(" : {}", self.sum.v()));
        });
    }
}

pub trait SumView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl SumView for SumsNode {
    fn view(&self, label: &str, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("A 1-second delay is applied to all addition");
            self.sum1.view(&format!("{label}.sum1"), ui);
            self.sum2.view(&format!("{label}.sum2"), ui);
            self.total.view(&format!("{label}.total"), ui);
        });
    }
}