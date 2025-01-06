use eframe::egui::Response;

mod inc_button;
mod inc_on_click;

pub use self::{
    inc_button::*,
    inc_on_click::*,
};

pub trait Clickable {
    fn clicked(&self) -> bool;
}

impl Clickable for Response {
    fn clicked(&self) -> bool { self.clicked() }   
}