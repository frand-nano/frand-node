use std::sync::Arc;
use crossbeam::channel::Sender;
use crate::bases::Packet;

#[derive(Clone)]
pub enum Reporter {
    Callback(Arc<dyn Fn(Packet) + Send + Sync>),
    Sender(Sender<Packet>),
}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reporter::Callback(_) => write!(f, "Reporter::Callback(Arc<dyn Fn(Packet) + Send + Sync>)"),
            Reporter::Sender(sender) => write!(f, "Reporter::Sender({:#?})", sender),
        }
    }
}

impl Reporter {
    pub fn new_callback<F>(callback: F) -> Self 
    where F: 'static + Fn(Packet) + Send + Sync { 
        Self::Callback(Arc::new(callback)) 
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

impl<F> From<F> for Reporter 
where F: 'static + Fn(Packet) + Send + Sync {
    fn from(callback: F) -> Self {
        Self::new_callback(callback)
    }
}