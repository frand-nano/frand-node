use std::time::Duration;
use serde::{Deserialize, Serialize};
use eframe::egui::*;
use frand_node::*;
use tokio::time::sleep;
use crate::widget::title_frame::TitleFrame;

use super::{SumMessage, Sum};

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Sums {
    pub values: Vec<Sum>,
    pub sums: Vec<i32>,
    pub total: i32,
}

impl System for Sums {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        use SumsMessage::*;
        use SumMessage::*;
        use VecMessage::*;

        match message {
            // values 에 push 또는 pop 이 emit 되면 sums 에 push 또는 pop 을 emit 하여 길이 동기화
            Values(Push(_)) => self.sums.emit_push(Default::default()),
            Values(Pop(_)) => self.sums.emit_pop(),

            // sums 에 push 또는 pop 이 emit 되면 values 에 push 또는 pop 을 emit 하여 길이 동기화
            Sums(Push(_)) => self.values.emit_push(Default::default()),
            Sums(Pop(_)) => self.values.emit_pop(),

            // values 의 index 번째 item 에 sum 이 emit 되었을 때
            // sums 의 index 번째 item 에 sum 을 emit
            Values(Item((index, Sum(sum)))) => self.sums.items()[index as usize].emit(sum),            

            // sums 에 emit 되었을 때
            // sums 의 모든 값들을 Box에 모아 1초뒤에 그 합을 emit
            Sums(_) => {
                let values: Box<_> = self.sums.items().map(|n| n.v()).collect();
                self.total.emit_future(async move {
                    sleep(Duration::from_millis(1000)).await;
                    values.iter().sum()
                })
            },       

            // 그 외의 메시지를 fallback 하여 전달
            // values: Vec<Sum> 
            // Sum Node 는 a, b, sum 을 가지며 a 또는 b 에 emit 되면 sum 에 그 합을 emit
            message => self.fallback(message, delta)
        }        
    }
}

impl Widget for &Sums {
    fn ui(self, ui: &mut Ui) -> Response {      
        ui.title_frame("Sums", |ui| {
            ui.label("A 1-second delay is applied to all addition");

            ui.horizontal(|ui| {
                let len = self.sums.len();
    
                if ui.button(format!(" - ")).clicked() {
                    self.sums.emit_pop();
                }
    
                ui.label(format!(" len: {len} "));
    
                if ui.button(format!(" + ")).clicked() {
                    self.sums.emit_push(0);
                }
            });

            for value in self.values.items() {   
                value.ui(ui);
            }

            ui.horizontal(|ui| {
                for sum in self.sums.items() {   
                    ui.label(format!("{} +", sum.v()));
                }
            });

            ui.label(format!("Total: {}", self.total.v()));
        }).response
    }
}