use serde::{Deserialize, Serialize};
use frand_node::ext::*;
use eframe::egui::*;
use crate::model::Sums;
use super::sum::{SumView, SumWidget};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct SumsView {
    pub model: Proxy<Sums>,
    pub values: Vec<SumView>,
}

impl System for SumsView {
    fn handle(
        node: Self::Node<'_>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    ) {
        use sums_view::Message::*;
        use vec::Message::*;

        match message {
            // values 에 push 또는 pop 이 emit 되면 sums 에 push 또는 pop 을 emit 하여 길이 동기화
            Values(Push(_)) => {
                if let Some(model) = node.model.subject() {
                    model.values.emit_push(Default::default())
                }
            },
            Values(Pop) => {
                if let Some(model) = node.model.subject() {
                    model.values.emit_pop()
                }
            },

            message => Self::fallback(node, message, delta),
        }        
    }
}

impl Widget for sums_view::Node<'_> {
    fn ui(self, ui: &mut Ui) -> Response {     
        ui.vertical(|ui| {
            if let Some(model) = self.model.subject() { 
                ui.horizontal(|ui| {
                    let len = model.values.len();
        
                    if ui.button(format!(" - ")).clicked() {
                        self.values.emit_pop();
                    }
        
                    ui.label(format!(" len: {len} "));
        
                    if ui.button(format!(" + ")).clicked() {
                        self.values.emit_push(Default::default());
                    }
                });
    
                for (model, view) in model.values.items().zip(self.values.items()) {   
                    ui.add(SumWidget(model.node(), view.node()));
                }
    
                ui.horizontal(|ui| {
                    for sum in model.sums.items() {   
                        ui.label(format!("{} +", sum.node().v()));
                    }
                });
    
                ui.label(format!("Total: {}", model.total.v()));
            } else {
                ui.label("The model is not applied.");
            }        
        }).response       
    }
}