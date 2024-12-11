use anyhow::Result;
use std::ops::Deref;
use bases::{Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver, SendError, Sender};
use crate::*;

pub struct Container<S: State> {
    node: S::Node,
    input_tx: Sender<Packet>,
    input_rx: Receiver<Packet>,
    output_tx: Sender<Packet>,
    output_rx: Option<Receiver<Packet>>,
}

impl<S: State> Deref for Container<S> {
    type Target = S::Node;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<S: State> Container<S> {
    pub fn node(&self) -> &S::Node { &self.node }
    pub fn input_tx(&self) -> &Sender<Packet> { &self.input_tx }
    pub fn output_tx(&self) -> &Sender<Packet> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<Receiver<Packet>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
    ) -> Self where F: 'static + Fn(Result<(), SendError<Packet>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded();
        let (output_tx, output_rx) = unbounded();

        let input_tx_clone = input_tx.clone();
        let callback = move |packet| {
            callback(input_tx_clone.send(packet))
        };

        Self {
            node: S::new_node(Reporter::new_callback(callback)),
            input_tx, input_rx,
            output_tx,
            output_rx: Some(output_rx),
        }
    }

    pub fn process(&mut self) -> Result<()> {
        Ok(while let Ok(packet) = self.input_rx.try_recv() {            
            self.node.apply(0, &packet)?;

            if self.output_rx.is_none() {
                self.output_tx.send(packet)?;
            }
        })        
    }
}