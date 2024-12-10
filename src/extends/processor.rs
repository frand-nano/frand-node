use anyhow::Result;
use std::{collections::HashSet, ops::Deref};
use bases::{NodeKey, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver};
use crate::*;

pub struct Processor<S: State> {
    state: S,
    input_node: S::Node,
    process_node: S::Node,
    processed: HashSet<NodeKey>,    
    input_rx: Receiver<Packet>,
    process_rx: Receiver<Packet>,
    update: fn(&S::StateNode<'_>, S::Message) -> Result<()>,
}

impl<S: State> Deref for Processor<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target { &self.state }
}

impl<S: State> Processor<S> {
    pub fn node(&self) -> &S::Node { &self.input_node }
    pub fn state_node(&mut self) -> S::StateNode<'_> { self.state.with(&self.input_node) }

    pub fn new<F>(
        callback: F,
        update: fn(&S::StateNode<'_>, S::Message) -> Result<()>,
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
            processed: HashSet::new(),
            input_rx,
            process_rx,
            update,
        }
    }

    pub fn process<'sn>(&'sn mut self) -> Result<()> {
        let mut state_node = self.state.with(&self.process_node);

        while let Ok(mut packet) = self.input_rx.try_recv() {
            loop {
                if !self.processed.contains(packet.key()) {
                    self.processed.insert(packet.key().clone());
        
                    let message = state_node.apply_export(0, &packet)?;
        
                    (self.update)(&state_node, message)?;
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