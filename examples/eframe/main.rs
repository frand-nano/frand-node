use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::Duration};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use eframe::{egui::*, Frame, NativeOptions};
use frand_node::prelude::*;
use tokio::{spawn, time::sleep};
use model::Model;
use view::{TitleFrame, View};

mod model;
mod view;

#[derive(Debug, Default)]
struct App {
    model: Component<Model>,
}

#[derive(Debug)]
struct Ui {
    frame_total: usize,
    model_output_total: Arc<AtomicUsize>,
    view_output_total: usize,
    view: Component<View>,
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {    
        CentralPanel::default().show(ctx, |ui| {
            ui.title_frame("Ui", |ui| {
                ui.label(format!("frame_total: {}", self.frame_total));
                ui.label(format!("model_output_total: {}", self.model_output_total.load(Ordering::Relaxed)));
                ui.label(format!("view_output_total: {}", self.view_output_total));
            });

            self.view.node().ui(ui);

            let output = self.view.try_update();

            if !output.is_empty() {
                self.view_output_total += output.len();
                ctx.request_repaint_after_secs(0.05);
            }

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
            let model_output_total = Arc::new(AtomicUsize::new(0));

            let mut app = App::default();
            let view: Component<View> = Component::default();

            view.node().stopwatch.model.set_subject(
                app.model.node().stopwatch,
            ).unwrap();

            view.node().sums.model.set_subject(
                app.model.node().sums,
            ).unwrap();

            let ui = Ui { 
                frame_total: 0,
                model_output_total: model_output_total.clone(),
                view_output_total: 0,
                view, 
            };

            spawn(
                async move {
                    loop {
                        sleep(Duration::from_millis(50)).await;

                        let output = app.model.update().await;

                        if !output.is_empty() {
                            model_output_total.fetch_add(output.len(), Ordering::Relaxed);
                            ctx.request_repaint();
                        }
                    }
                }
            );

            Ok(Box::new(ui))
        }),
    )
}