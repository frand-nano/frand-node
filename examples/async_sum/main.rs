use eframe::{egui::{CentralPanel, Context}, CreationContext, Frame};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::runtime::Runtime;
use frand_node::*;
use sum::*;
use view::*;

mod sum;
mod view;

struct App {    
    node: SumsNode,
}

impl App {
    fn new(runtime: &Runtime, cc: &CreationContext) -> Self {
        let mut processor = AsyncProcessor::<Sums>::new(
            |result| if let Err(err) = result { log::info!("{err}") }, 
            |node, message| node.handle(message),
        );

        let node = processor.node().clone();

        let ctx = cc.egui_ctx.clone();
        runtime.spawn(async move {
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

fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let runtime = Runtime::new().unwrap();
    let options = eframe::NativeOptions::default();
    
    eframe::run_native(
        "AsyncSum",
        options,
        Box::new(|cc| Ok(Box::new(App::new(&runtime, cc)))),
    )
}