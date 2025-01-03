use std::fmt::Display;
use eframe::egui::Response;
use num::Integer;
use frand_node::*;
use super::Clickable;

#[allow(dead_code)]
pub trait IncOnClick<M: Message, S: State>: Clickable 
where S: Display + Integer {
    fn inc_on_click(&self, node: &impl Node<M, S>) {     
        if self.clicked() {
            let mut value = node.clone_state();
            value.inc();

            node.emit(value);
        }
    }
}

impl<M: Message, S: State> IncOnClick<M, S> for Response 
where S: Display + Integer {}