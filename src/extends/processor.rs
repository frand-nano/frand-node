use anyhow::Result;
use std::collections::HashSet;
use bases::{NodeKey, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver};
use crate::*;

pub struct Processor<S: State> {
    state: S,
    input_node: S::Node,
    process_node: S::Node,
    update: fn(&S, &S::Node, S::Message) -> Result<()>,
    processed: HashSet<NodeKey>,    
    input_rx: Receiver<Packet>,
    process_rx: Receiver<Packet>,
}

impl<S: State> Processor<S> {
    pub fn state(&self) -> &S { &self.state }
    pub fn node(&self) -> &S::Node { &self.input_node }

    pub fn new<F>(
        callback: F,
        update: fn(&S, &S::Node, S::Message) -> Result<()>,
    ) -> Self where F: 'static + Fn() {
        let (input_tx, input_rx) = unbounded();
        let (process_tx, process_rx) = unbounded();

        let callback = move |packet| {
            input_tx.send(packet).unwrap();
            callback()
        };

        Self {
            state: S::default(),
            input_node: S::new_node(Reporter::new_callback(callback)),
            process_node: S::new_node(Reporter::new_sender(process_tx)),
            update,
            processed: HashSet::new(),
            input_rx,
            process_rx,
        }
    }

    pub fn process(&mut self) -> Result<()> {
        while let Ok(mut packet) = self.input_rx.try_recv() {
            loop {
                if !self.processed.contains(packet.key()) {
                    self.processed.insert(packet.key().clone());
        
                    let message = S::Message::from_packet(0, &packet)?;
        
                    self.state.apply(0, &packet)?;
        
                    (self.update)(&self.state, &self.process_node, message)?;
                }
                match self.process_rx.try_recv() {
                    Ok(next) => packet = next,
                    _ => break,
                }
            }
                
            self.processed.clear(); 
        }        
    
        Ok(())
    }
}