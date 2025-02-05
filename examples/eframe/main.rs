use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::Duration};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use eframe::{egui::*, Frame, NativeOptions};
use frand_node::prelude::*;
use model::Model;
use tokio::{spawn, time::sleep};
use widget::title_frame::TitleFrame;

mod model;
mod widget;

#[derive(Debug, Default)]
struct App {
    model: Component<Model>,
}

#[derive(Debug)]
struct Ui {
    frame_total: usize,
    output_total: Arc<AtomicUsize>,
    model: Consensus<Model>,
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {    
        CentralPanel::default().show(ctx, |ui| {
            ui.title_frame("Ui", |ui| {
                ui.label(format!("frame_total: {}", self.frame_total));
                ui.label(format!("output_total: {}", self.output_total.load(Ordering::Relaxed)));
            });

            self.model.read().node().ui(ui);

            self.frame_total += 1;
        });
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {    
    TermLogger::init(log::LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
    .unwrap_or_else(|err| log::error!("{err}"));

    eframe::run_native(
        "Examples",
        NativeOptions::default(),
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();
            let output_total = Arc::new(AtomicUsize::new(0));

            let mut app = App::default();
            let ui = Ui { 
                frame_total: 0,
                output_total: output_total.clone(),
                model: app.model.clone(), 
            };

            spawn(
                async move {
                    loop {
                        sleep(Duration::from_millis(50)).await;

                        let output = app.model.update().await;

                        if !output.is_empty() {
                            output_total.fetch_add(output.len(), Ordering::Relaxed);
                            ctx.request_repaint();
                        }
                    }
                }
            );

            Ok(Box::new(ui))
    }),
    )
}