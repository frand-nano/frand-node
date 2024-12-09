use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use crate::bases::Packet;
use super::{Callback, PacketSender, Sender};

#[derive(Clone)]
pub enum Reporter {
    Callback(Callback),
    Sender(Sender),
}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reporter::Callback(_) => write!(f, "Reporter::Callback"),
            Reporter::Sender(sender) => write!(f, "Reporter::Sender({:#?})", sender),
        }
    }
}

impl Reporter {
    pub fn new_callback<F>(callback: F) -> Self where F: 'static + FnMut(Packet) { 
        Self::Callback(Rc::new(RefCell::new(callback))) 
    }

    pub fn new_sender(sender: PacketSender) -> Self { 
        Self::Sender(sender.into()) 
    }

    pub fn report(&self, packet: Packet) -> Result<()> {
        Ok(match self {
            Reporter::Callback(callback) => callback.borrow_mut()(packet),
            Reporter::Sender(sender) => sender.send(packet)?,
        })
    }
}