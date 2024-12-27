use std::fmt::Display;
use eframe::egui::Ui;
use num::Integer;
use frand_node::*;
use super::*;

#[allow(dead_code)]
pub trait IncButton<M: Message, S: State> 
where S: Display + Integer {
    fn inc_button(&mut self, node: &impl Node<M, S>);
}

impl<M: Message, S: State> IncButton<M, S> for Ui 
where S: Display + Integer {
    fn inc_button(&mut self, node: &impl Node<M, S>) {
        let value = node.clone_state();
        
        self.button(format!(" {value} "))
        .inc_on_click(node);
    }
}