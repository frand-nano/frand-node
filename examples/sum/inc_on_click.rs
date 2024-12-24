use std::fmt::Display;
use eframe::egui::{Response, Ui};
use frand_node::*;
use num::Integer;

pub trait IncOnClick<S: State, N: Node<S>> 
where S: Display + Integer {
    fn inc_on_click(&self, node: &impl AsRef<N>);
}

pub trait IncButton<S: State, N: Node<S>> 
where S: Display + Integer {
    fn inc_button(&mut self, node: &impl AsRef<N>);
}

impl<S: State, N: Node<S>> IncOnClick<S, N> for Response 
where S: Display + Integer {
    fn inc_on_click(&self, node: &impl AsRef<N>) {        
        if self.clicked() {
            let node = node.as_ref();

            let mut value = node.clone_state();
            value.inc();

            node.emit(value);
        }
    }
}

impl<S: State, N: Node<S>> IncButton<S, N> for Ui 
where S: Display + Integer {
    fn inc_button(&mut self, node: &impl AsRef<N>) {
        let node = node.as_ref();
        let value = node.clone_state();
        
        self.button(format!(" {value} "))
        .inc_on_click(node);
    }
}