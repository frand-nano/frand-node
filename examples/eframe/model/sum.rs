use tokio::time::sleep;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use frand_node::prelude::*;
use eframe::egui::*;
use crate::widget::clickable::IncButton;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Sum {
    pub a: u32,
    pub b: u32,
    pub sum: u32,
}

impl System for Sum {
    fn handle(
        node: Self::Node<'_>, 
        message: Self::Message, 
        delta: Option<std::time::Duration>,
    ) {        
        use sum::Message::*;
        
        match message {
            // a 또는 b 가 emit 되면 sum1.sum 에 sum1.a + sum1.b 를 emit          
            A(_) | B(_) => {
                let sum = node.a.v() + node.b.v();
                node.sum.emit_future(async move {
                    // 적용 전 1초 비동기 대기
                    sleep(Duration::from_millis(1000)).await;
                    sum 
                })
            },
            
            // 그 외의 메시지를 fallback 하여 전달
            message => Self::fallback(node, message, delta),
        }          
    }
}

impl Widget for sum::Node<'_> {
    fn ui(self, ui: &mut Ui) -> Response {       
        ui.horizontal(|ui| {
            ui.inc_button(self.a);
            ui.label(" + ");
            ui.inc_button(self.b);
            ui.label(format!(" : {}", self.sum.v()));
        }).response   
    }
}
