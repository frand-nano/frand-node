use eframe::{egui::{CentralPanel, Context}, Frame};
use frand_node::*;
use sum::*;

mod sum;

struct App {
    processor: Processor::<Sum>,
}

impl App {
    fn new() -> Self {
        Self { processor: Processor::new(
            || {}, 
            |node, message| {
                use SumMessage::*;
                use SumSubMessage::*;

                match message {
                    sum1(a(_) | b(_)) => node.sum1.emit_sum(),
                    sum1(sum(s)) => node.sum3.a.emit(s),

                    sum2(a(_) | b(_)) => node.sum2.emit_sum(),
                    sum2(sum(s)) => node.sum3.b.emit(s),

                    sum3(a(_) | b(_)) => node.sum3.emit_sum(),

                    _ => (),
                }
            },
        ) }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let node = self.processor.state_node();

        CentralPanel::default().show(ctx, |ui| {
            node.view("sum", ui);
        });

        self.processor.process().unwrap();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    
    eframe::run_native(
        "EframeSum",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}