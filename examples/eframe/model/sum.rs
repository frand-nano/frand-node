use std::time::Duration;
use serde::{Deserialize, Serialize};
use eframe::egui::*;
use frand_node::*;
use tokio::time::sleep;
use crate::widget::clickable::IncButton;

#[node]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Sum {
    pub a: i32,
    pub b: i32,
    pub sum: i32,
}

impl System for Sum {
    fn handle(&self, message: Self::Message, delta: Option<f32>) {
        use SumMessage::*;

        // Message 를 match 하여 이벤트 처리
        match message {
            //a 또는 b 가 emit 되면 sum1.sum 에 sum1.a + sum1.b 를 emit
            A(_) | B(_) => self.emit_expensive_sum(),

            // 그 외의 메시지를 fallback 하여 전달
            message => self.fallback(message, delta)
        }        
    }
}

impl Sum {
    // Sum 의 a 와 b 의 합을 sum 에 emit()
    fn emit_expensive_sum(&self) {
        let a = self.a.v();
        let b = self.b.v();

        // emit_future 를 사용하여 1초 대기 후 합 emit
        self.sum.emit_future(async move { 
            sleep(Duration::from_millis(1000)).await;
            a + b 
        })
    }
}

impl Widget for &Sum {
    fn ui(self, ui: &mut Ui) -> Response {       
        ui.horizontal(|ui| {
            ui.inc_button(&self.a);
            ui.label(" + ");
            ui.inc_button(&self.b);
            ui.label(format!(" : {}", self.sum.v()));
        }).response   
    }
}