use eframe::{egui::{CentralPanel, Context}, CreationContext, Frame};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tokio::{runtime::Runtime, sync::mpsc::{unbounded_channel, UnboundedReceiver}};
use frand_node::*;
use sum::*;
use view::*;

mod sum;
mod view;

struct App {    
    state: Sums,
    anchor: SumsAnchor,
    view_rx: UnboundedReceiver<Packet>,
}

impl App {
    fn new(runtime: &Runtime, cc: &CreationContext) -> Self {
        let mut processor = AsyncProcessor::<Sums>::new(
            |result| if let Err(err) = result { log::info!("{err}") }, 
            |node, message| node.handle(message),
        );

        let state = processor.state().clone();
        let anchor = processor.anchor().clone();
        let (view_tx, view_rx) = unbounded_channel();

        let ctx = cc.egui_ctx.clone();
        runtime.spawn(async move {
            let mut processor_output_rx = processor.take_output_rx().unwrap();
            processor.start().await;
            
            while let Some(packet) = processor_output_rx.recv().await {
                view_tx.send(packet).unwrap();
                ctx.request_repaint();
            }
        });
        
        Self { 
            state, 
            anchor, 
            view_rx, 
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {    
        while let Ok(packet) = self.view_rx.try_recv() {
            self.state.apply(0, packet).unwrap();
        }

        let node = self.state.with(&self.anchor);

        CentralPanel::default().show(ctx, |ui| {
            node.view("sum", ui);
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