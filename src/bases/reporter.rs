use std::rc::Rc;
use crossbeam::channel::Sender;
use crate::bases::Packet;

#[derive(Clone)]
pub enum Reporter {
    Callback(Rc<dyn Fn(Packet)>),
    Sender(Sender<Packet>),
}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reporter::Callback(_) => write!(f, "Reporter::Callback(Rc<dyn Fn(Packet)>)"),
            Reporter::Sender(sender) => write!(f, "Reporter::Sender({:#?})", sender),
        }
    }
}

impl Reporter {
    pub fn new_callback<F>(callback: F) -> Self where F: 'static + Fn(Packet) { 
        Self::Callback(Rc::new(callback)) 
    }

    pub fn new_sender(sender: Sender<Packet>) -> Self { 
        Self::Sender(sender) 
    }

    pub fn report(&self, packet: Packet) {
        match self {
            Reporter::Callback(callback) => callback(packet),
            Reporter::Sender(sender) => sender.send(packet)
            .unwrap_or_else(|err| log::trace!("{err}")),
        }
    }
}