use std::{fmt::Display, ops::Add};
use anyhow::Result;
use eframe::egui::Ui;
use serde::{Deserialize, Serialize};
use frand_node::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Sum {
    pub sum1: SumSub,
    pub sum2: SumSub,
    pub sum3: SumSub,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct SumSub {
    pub a: i32,
    pub b: i32,
    pub sum: i32,
}

impl SumSubStateNode<'_> {
    pub fn emit_sum(&self) -> Result<()> {
        self.sum.emit(*self.a + *self.b)
    }
}

pub trait IncButton<'a, S: State>: StateNode<'a, S> 
where S: Display + Add<i32, Output = S> {
    fn view(&self, ui: &mut Ui) {
        let value = self.clone_state();

        if ui.button(format!(" {value} ")).clicked() {
            self.emit(value + 1).unwrap();
        }
    }
}

impl<'a> IncButton<'a, i32> for <i32 as State>::StateNode<'a> {}

pub trait SumSubView {
    fn view(&self, label: &str, ui: &mut Ui);
}

impl SumSubView for SumSubStateNode<'_> {
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

impl SumView for SumStateNode<'_> {
    fn view(&self, label: &str, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.sum1.view(&format!("{label}.sum1"), ui);
            self.sum2.view(&format!("{label}.sum2"), ui);
            self.sum3.view(&format!("{label}.sum3"), ui);
        });
    }
}