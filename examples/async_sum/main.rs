use eframe::{egui::{CentralPanel, Context}, CreationContext, Frame, NativeOptions};
use extends::Processor;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use frand_node::*;
use sum::*;
use view::*;

mod clickable;
mod sum;
mod view;

struct App {    
    node: SumsNode<SumsMessage>,
}

impl App {
    fn new(cc: &CreationContext) -> Self {
        let mut processor = Processor::<Sums>::new(
            |result| if let Err(err) = result { log::error!("{err}") }, 
            |node, message| node.handle(message),
        );

        let node = processor.node().clone();

        let ctx = cc.egui_ctx.clone();
        tokio::spawn(async move {
            let mut processor_output_rx = processor.take_output_rx().unwrap();

            processor.start().await;
            
            while let Some(_) = processor_output_rx.recv().await {
                ctx.request_repaint();
            }
        });
        
        Self { node }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {    
        CentralPanel::default().show(ctx, |ui| {
            self.node.view("sum", ui);
        });
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::warn!("{err}"));

    eframe::run_native(
        "AsyncSum",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}