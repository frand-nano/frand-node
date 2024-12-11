use eframe::{egui::{CentralPanel, Context}, Frame};
use frand_node::*;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use sum::*;
use view::*;

mod sum;
mod view;
mod test;

struct App {
    processor: Processor::<Sums>,
}

impl App {
    fn new() -> Self {
        Self { processor: Processor::new(
            |result| if let Err(err) = result { log::info!("{err}") }, 
            |node, message| node.handle(message),
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
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();
    
    let options = eframe::NativeOptions::default();
    
    eframe::run_native(
        "EframeSum",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}