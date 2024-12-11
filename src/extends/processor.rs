use anyhow::Result;
use std::{collections::HashSet, ops::Deref};
use bases::{NodeKey, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver, SendError, Sender};
use crate::*;

pub struct Processor<S: State> {
    node: S::Node,
    process_node: S::Node,
    processed: HashSet<NodeKey>,    
    input_tx: Sender<Packet>,
    input_rx: Receiver<Packet>,
    process_rx: Receiver<Packet>,
    output_tx: Sender<Packet>,
    output_rx: Option<Receiver<Packet>>,
    update: fn(&S::Node, S::Message),
}

impl<S: State> Deref for Processor<S> {
    type Target = S::Node;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<S: State> Processor<S> {
    pub fn node(&self) -> &S::Node { &self.node }
    pub fn input_tx(&self) -> &Sender<Packet> { &self.input_tx }
    pub fn input_rx(&self) -> &Receiver<Packet> { &self.input_rx }
    pub fn output_tx(&self) -> &Sender<Packet> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<Receiver<Packet>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
        update: fn(&S::Node, S::Message),
    ) -> Self where F: 'static + Fn(Result<(), SendError<Packet>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded();
        let (process_tx, process_rx) = unbounded();
        let (output_tx, output_rx) = unbounded();

        let input_tx_clone = input_tx.clone();
        let callback = move |packet| {
            callback(input_tx_clone.send(packet))
        };

        let node = S::new_node(Reporter::new_callback(callback));

        Self {
            process_node: S::new_node_from(&node, Reporter::new_sender(process_tx)),
            node,
            processed: HashSet::new(),
            input_tx, input_rx,
            process_rx,
            output_tx,
            output_rx: Some(output_rx),
            update,
        }
    }

    pub fn process(&mut self) -> Result<()> {
        Ok(while let Ok(mut packet) = self.input_rx.try_recv() {
            loop {
                if !self.processed.contains(packet.key()) {
                    self.processed.insert(packet.key().clone());
        
                    let message = self.node.apply_export(0, &packet)?;

                    if self.output_rx.is_none() {
                        self.output_tx.send(packet)?;
                    }
        
                    (self.update)(&self.process_node, message);
                }
                match self.process_rx.try_recv() {
                    Ok(next) => packet = next,
                    _ => break,
                }
            }
                
            self.processed.clear(); 
        })     
    }
}