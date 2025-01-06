use std::fmt::Display;
use eframe::egui::*;
use frand_node::*;
use num::Integer;
use super::IncOnClick;

pub trait IncButton<S: State> 
where S: Display + Integer {
    fn inc_button(
        &mut self, 
        node: &impl Node<S>, 
    ) -> Response;
}

impl<S: State> IncButton<S> for Ui 
where S: Display + Integer {
    fn inc_button(
        &mut self, 
        node: &impl Node<S>, 
    ) -> Response {
        let value = node.clone_state();
        
        self.button(format!(" {value} "))
        .inc_on_click(node)
    }
}