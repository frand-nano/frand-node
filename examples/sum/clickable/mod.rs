mod num_button;
mod inc_on_click;

use eframe::egui::Response;

#[allow(unused_imports)]
pub use self::{
    num_button::*,
    inc_on_click::*,
};

pub trait Clickable {
    fn clicked(&self) -> bool;
}

impl Clickable for Response {
    fn clicked(&self) -> bool { self.clicked() }   
}