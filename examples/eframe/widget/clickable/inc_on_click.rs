use std::fmt::Display;
use eframe::egui::*;
use frand_node::*;
use num::Integer;
use super::Clickable;

pub trait IncOnClick<'n, S: System, N: Node<'n, S>>: Clickable + Sized 
where S: Display + Integer + Copy {
    fn inc_on_click(self, node: N) -> Self {     
        if self.clicked() {
            let mut value = node.v();
            
            value.inc();

            node.emit(value);
        }

        self
    }
}

impl<'n, S: System, N: Node<'n, S>> IncOnClick<'n, S, N> for Response 
where S: Display + Integer + Copy {}