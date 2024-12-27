use std::{collections::HashSet, ops::Deref};
use bases::{PacketMessage, NodeKey, Reporter, Result};
use crossbeam::channel::{unbounded, Receiver, SendError, Sender};
use crate::*;

pub struct Processor<S: State> {
    node: S::Node<S::Message>,
    consensus: S::Consensus<S::Message>,
    process_node: S::Node<S::Message>,
    processed: HashSet<NodeKey>,    
    input_tx: Sender<PacketMessage<S::Message>>,
    input_rx: Receiver<PacketMessage<S::Message>>,
    process_rx: Receiver<PacketMessage<S::Message>>,
    output_tx: Sender<PacketMessage<S::Message>>,
    output_rx: Option<Receiver<PacketMessage<S::Message>>>,
    update: fn(&S::Node<S::Message>, S::Message),
}

impl<S: State> Deref for Processor<S> {
    type Target = S::Node<S::Message>;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<S: State> Processor<S> {
    pub fn node(&self) -> &S::Node<S::Message> { &self.node }
    pub fn input_tx(&self) -> &Sender<PacketMessage<S::Message>> { &self.input_tx }
    pub fn input_rx(&self) -> &Receiver<PacketMessage<S::Message>> { &self.input_rx }
    pub fn output_tx(&self) -> &Sender<PacketMessage<S::Message>> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<Receiver<PacketMessage<S::Message>>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
        update: fn(&S::Node<S::Message>, S::Message),
    ) -> Self where F: 'static + Fn(Result<(), SendError<PacketMessage<S::Message>>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded();
        let (process_tx, process_rx) = unbounded();
        let (output_tx, output_rx) = unbounded();

        let input_tx_clone = input_tx.clone();
        let callback = move |message| {
            callback(input_tx_clone.send(message))
        };

        let consensus = S::Consensus::default();

        Self {
            node: consensus.new_node(&Reporter::new_callback(callback)),
            process_node: consensus.new_node(&Reporter::new_sender(process_tx)),
            consensus,
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
                if !self.processed.contains(&packet.header) {
                    self.processed.insert(packet.header.clone());
        
                    self.consensus.apply(packet.message.clone());

                    if self.output_rx.is_none() {
                        self.output_tx.send(packet.clone())?;
                    }
        
                    (self.update)(&self.process_node, packet.message);
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