use bases::MessageError;
use frand_node::*;
use eframe::{egui::*, CreationContext, Frame, NativeOptions};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::select;
use model::Model;
use view::View;

mod model;
mod view;
mod widget;

struct Ui {
    view: View,
}

impl Ui {
    fn new(cc: &CreationContext) -> Self {
        let mut model = Processor::<Model>::new(
            MessageError::log_error,
            System::handle,
        );

        let mut view = View::default();
        view.stopwatch.set_subject(&model.stopwatch);
        view.sums.set_subject(&model.sums);
        
        let mut view = Processor::<View>::new_with(
            view,
            MessageError::log_error,
            System::handle,
        );       

        let view_node = view.node().clone();

        let ctx = cc.egui_ctx.clone();
        tokio::spawn(async move {
            let mut model_output_rx = model.take_output_rx().unwrap();
            let mut view_output_rx = view.take_output_rx().unwrap();

            model.start(0.05).await;
            view.start(0.05).await;
            
            loop {
                select! {
                    Some(_) = model_output_rx.recv() => {
                        ctx.request_repaint();
                    }
                    Some(_) = view_output_rx.recv() => {
                        ctx.request_repaint();                        
                    }
                    else => { break; }
                }
            }        
        });
        
        Self { 
            view: view_node, 
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {    
        CentralPanel::default().show(ctx, |ui| {
            self.view.ui(ui);
        });
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::info!("{err}"));

    eframe::run_native(
        "Examples",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(Ui::new(cc)))),
    )
}