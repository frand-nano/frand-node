use anyhow::Result;
use std::{collections::HashSet, ops::Deref};
use bases::{AnchorKey, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver, SendError};
use crate::*;

pub struct Processor<S: State> {
    state: S,
    input_anchor: S::Anchor,
    process_anchor: S::Anchor,
    processed: HashSet<AnchorKey>,    
    input_rx: Receiver<Packet>,
    process_rx: Receiver<Packet>,
    messages: Vec<S::Message>,
    update: fn(&S::Node<'_>, S::Message),
}

impl<S: State> Deref for Processor<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target { &self.state }
}

impl<S: State> Processor<S> {
    pub fn state(&self) -> &S { &self.state }
    pub fn anchor(&self) -> &S::Anchor { &self.input_anchor }
    pub fn new_node(&self) -> S::Node<'_> { self.state.with(&self.input_anchor) }

    pub fn new<F>(
        callback: F,
        update: fn(&S::Node<'_>, S::Message),
    ) -> Self where F: 'static + Fn(Result<(), SendError<Packet>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded();
        let (process_tx, process_rx) = unbounded();

        let callback = move |packet| {
            callback(input_tx.send(packet))
        };

        Self {
            state: S::default(),
            input_anchor: S::new_anchor(Reporter::new_callback(callback)),
            process_anchor: S::new_anchor(Reporter::new_sender(process_tx)),
            processed: HashSet::new(),
            input_rx,
            process_rx,
            messages: Vec::new(),
            update,
        }
    }

    pub fn process<'sn>(&'sn mut self) -> Result<()> {
        let mut node = self.state.with(&self.process_anchor);

        while let Ok(mut packet) = self.input_rx.try_recv() {
            loop {
                if !self.processed.contains(packet.key()) {
                    self.processed.insert(packet.key().clone());
        
                    let message = node.apply_export(0, &packet)?;
                    self.messages.push(message.clone());
        
                    (self.update)(&node, message);
                }
                match self.process_rx.try_recv() {
                    Ok(next) => packet = next,
                    _ => break,
                }
            }
                
            self.processed.clear(); 
        }        

        drop(node);

        for message in self.messages.drain(..) {
            self.state.apply(message);
        }
    
        Ok(())
    }
}