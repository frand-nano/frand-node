use std::time::Duration;
use serde::{Deserialize, Serialize};
use frand_node::{prelude::*, vec::vec};
use eframe::egui::*;
use tokio::time::sleep;
use super::sum::{sum, Sum};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
pub struct Sums {
    pub values: Vec<Sum>,
    pub sums: Vec<u32>,
    pub total: u32,
}

impl System for Sums {
    fn handle<CS: System>(
        node: Self::Node<'_, CS>, 
        message: &Self::Message, 
        delta: Option<std::time::Duration>,
    ) {        
        use sums::Message::*;
        use sum::Message::*;
        use vec::Message::*;

        match message {
            // values 에 push 또는 pop 이 emit 되면 sums 에 push 또는 pop 을 emit 하여 길이 동기화
            Values(Push(item)) => node.sums.emit_push(item.sum),
            Values(Pop) => node.sums.emit_pop(),

            // values 의 index 번째 item 에 sum 이 emit 되었을 때
            // sums 의 index 번째 item 에 sum 을 emit
            Values(Item(index, Sum(sum))) => {
                node.sums.item(*index).emit(*sum)
            },            

            // sums 에 emit 되었을 때
            // sums 의 모든 값들을 Box에 모아 1초뒤에 그 합을 emit
            Sums(_) => {
                let values: Box<_> = node.sums.items().map(|n| *n).collect();
                node.total.emit_future(async move {
                    sleep(Duration::from_millis(1000)).await;
                    values.iter().sum()
                })
            },       

            // 그 외의 메시지를 fallback 하여 전달
            // values: Vec<Sum> 
            // Sum Node 는 a, b, sum 을 가지며 a 또는 b 에 emit 되면 sum 에 그 합을 emit
            message => Self::fallback(node, message, delta),
        }             
    }
}

impl<CS: System> Widget for sums::Node<'_, CS> {
    fn ui(self, ui: &mut Ui) -> Response {       
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let len = self.values.len();
    
                if ui.button(format!(" - ")).clicked() {
                    self.values.emit_pop();
                }
    
                ui.label(format!(" len: {len} "));
    
                if ui.button(format!(" + ")).clicked() {
                    self.values.emit_push(Default::default());
                }
            });

            for value in self.values.items() {   
                value.node().ui(ui);
            }

            ui.horizontal(|ui| {
                for sum in self.sums.items() {   
                    ui.label(format!("{} +", *sum.node()));
                }
            });

            ui.label(format!("Total: {}", *self.total));
        }).response
    }
}
