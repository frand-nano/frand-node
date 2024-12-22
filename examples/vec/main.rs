use eframe::{egui::{CentralPanel, Context}, Frame, NativeOptions};
use frand_node::*;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use view::*;

mod inc_button;
mod view;

struct App {
    container: Container<Vec<i32>>,
}

impl App {
    fn new() -> Self {
        Self { container: Container::<Vec<i32>>::new(
            |result| if let Err(err) = result { log::error!("{err}") }, 
        ) }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.container.view(ui);
        });

        self.container.process().unwrap();
    }
}

fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::warn!("{err}"));
    
    eframe::run_native(
        "Vec",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}