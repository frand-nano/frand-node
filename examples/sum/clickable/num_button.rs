use std::fmt::Display;
use eframe::egui::Ui;
use num::Integer;
use frand_node::*;
use super::*;

#[allow(dead_code)]
pub trait IncButton<S: State> 
where S: Display + Integer {
    fn inc_button(&mut self, node: &impl Node<S>);
}

impl<S: State> IncButton<S> for Ui 
where S: Display + Integer {
    fn inc_button(&mut self, node: &impl Node<S>) {
        let value = node.clone_state();
        
        self.button(format!(" {value} "))
        .inc_on_click(node);
    }
}