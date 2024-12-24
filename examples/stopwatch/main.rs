use std::{thread::{sleep, spawn}, time::{Duration, Instant}};
use eframe::{egui::{CentralPanel, Context}, CreationContext, Frame, NativeOptions};
use extends::Processor;
use frand_node::*;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use stopwatch::*;
use view::*;

mod stopwatch;
mod view;

struct App {
    processor: Processor<Stopwatch>,
}

impl App {
    fn new(cc: &CreationContext) -> Self {
        let processor = Processor::<Stopwatch>::new(
            |result| if let Err(err) = result { log::error!("{err}") }, 
            |node, message| node.handle(message),
        );

        let ctx = cc.egui_ctx.clone();
        let node = processor.node().clone();

        spawn(move || {
            let mut last = Instant::now();
            loop {
                let now = Instant::now();

                if let Some(delta) = now.checked_duration_since(last) {
                    node.delta.emit(delta.as_secs_f32());
                    ctx.request_repaint();
                }

                last = now;

                sleep(Duration::from_millis(50));
            }
        });
        
        Self { processor }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {    
        CentralPanel::default().show(ctx, |ui| {
            self.processor.view(ui);
        });

        self.processor.process().unwrap();
    }
}

fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::warn!("{err}"));

    eframe::run_native(
        "Stopwatch",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}