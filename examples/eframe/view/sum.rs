use serde::{Deserialize, Serialize};
use frand_node::ext::*;
use eframe::egui::*;
use crate::model::Sum;
use super::{FillGlow, Glow, OnClickGlow};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct SumView {
    pub a: Glow,
    pub b: Glow,
}

impl System for SumView {}

pub struct SumWidget<'n>(
    pub <Sum as State>::Node<'n>, 
    pub <SumView as State>::Node<'n>,
);

impl Widget for SumWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {  
        let Self(model, view) = self;

        ui.horizontal(|ui| {      
            ui.add(GlowIncButton(model.a, view.a));        
            ui.label(" + ");
            ui.add(GlowIncButton(model.b, view.b));   
            ui.label(format!(" : {}", model.sum.v()));    
        }).response       
    }
}

pub struct GlowIncButton<'n>(
    pub <u32 as State>::Node<'n>, 
    pub <Glow as State>::Node<'n>,
);

impl Widget for GlowIncButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {  
        let Self(number, glow) = self;

        let value = number.clone_state().unwrap();

        let button = ui.add(
            Button::new(
                format!(" {} ", value)
            )
            .fill_glow(
                &glow, 
                Color32::from_gray(50), 
                Color32::from_gray(200),
            )
        );

        if button.clicked() {
            number.emit(value + 1);
        }

        button.on_click_glow(&glow, 0.5)
    }
}