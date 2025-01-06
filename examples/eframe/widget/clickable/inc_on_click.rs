use std::fmt::Display;
use eframe::egui::*;
use frand_node::*;
use num::Integer;
use super::Clickable;

pub trait IncOnClick<S: State>: Clickable + Sized 
where S: Display + Integer {
    fn inc_on_click(self, node: &impl Node<S>) -> Self {     
        if self.clicked() {
            let mut value = node.clone_state();
            value.inc();

            node.emit(value);
        }

        self
    }
}

impl<S: State> IncOnClick<S> for Response 
where S: Display + Integer {}