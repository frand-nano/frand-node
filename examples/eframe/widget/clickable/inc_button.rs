use std::fmt::Display;
use eframe::egui::*;
use frand_node::*;
use num::Integer;
use super::IncOnClick;

pub trait IncButton<'n, S: System, N: Node<'n, S>> 
where S: Display + Integer + Copy {
    fn inc_button(&mut self, node: N) -> Response;
}

impl<'n, S: System, N: Node<'n, S>> IncButton<'n, S, N> for Ui 
where S: Display + Integer + Copy {
    fn inc_button(&mut self, node: N) -> Response {
        let value = *node;
        
        self.button(format!(" {value} "))
        .inc_on_click(node)
    }
}