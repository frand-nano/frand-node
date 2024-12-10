use std::{thread::{sleep, spawn}, time::{Duration, Instant}};
use eframe::{egui::{CentralPanel, Context}, CreationContext, Frame};
use frand_node::*;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use timer::*;

mod timer;

struct App {
    processor: Processor<Timer>,
}

impl App {
    fn new(cc: &CreationContext) -> Self {
        let processor = Processor::<Timer>::new(
            |result| if let Err(err) = result { log::info!("{err}") }, 
            Timer::update,
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
        let node = self.processor.state_node();

        CentralPanel::default().show(ctx, |ui| {
            node.view(ui);
        });

        self.processor.process().unwrap();
    }
}

fn main() -> eframe::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let options = eframe::NativeOptions::default();
    
    eframe::run_native(
        "EframeTimer",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}