use eframe::{egui::{CentralPanel, Context}, Frame, NativeOptions};
use extends::Processor;
use frand_node::*;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use sum::*;
use view::*;

mod inc_on_click;
mod sum;
mod view;
mod test;

struct App {
    processor: Processor<Sums>,
}

impl App {
    fn new() -> Self {
        Self { processor: Processor::<Sums>::new(
            |result| if let Err(err) = result { log::error!("{err}") }, 
            |node, message| node.handle(message),
        ) }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.processor.view("sum", ui);
        });

        self.processor.process().unwrap();
    }
}

fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::warn!("{err}"));
    
    eframe::run_native(
        "Sum",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}