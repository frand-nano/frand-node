use anyhow::Result;
use std::collections::HashSet;
use bases::{NodeKey, Reporter};
use crossbeam::channel::{unbounded, Receiver, Sender};
use crate::*;
use super::Packet;

pub struct Processor<S: State> {
    state: S,
    node: S::Node,
    output_rx: Receiver<Packet>,
}

impl<S: State> Processor<S> {
    pub fn state(&self) -> &S { &self.state }
    pub fn node(&self) -> &S::Node { &self.node }

    pub fn new(update: fn(&S, &S::Node, S::Message) -> Result<()>) -> Self {
        let (node_tx, node_rx) = unbounded();
        let mut state = S::default();
        let node = S::new_node(Reporter::new_sender(node_tx));
        let mut processed = HashSet::new();
        let (output_tx, output_rx) = unbounded();

        let callback = move |packet| {
            process(&mut state, &node, &mut processed, &node_rx, packet, update, &output_tx)
            .unwrap_or_else(|err| log::error!("{err}"));
        };

        Self {
            state: S::default(),
            node: S::new_node(Reporter::new_callback(callback)),
            output_rx,
        }
    }

    pub fn process(&mut self) -> Result<()> {
        Ok(while let Ok(packet) = self.output_rx.try_recv() {
            self.state.apply(0, &packet)?;
        })
    }
}

fn process<S: State>(
    state: &mut S,
    node: &S::Node,
    processed: &mut HashSet<NodeKey>,
    node_rx: &Receiver<Packet>,
    mut packet: Packet,
    update: fn(&S, &S::Node, S::Message) -> Result<()>,
    output_tx: &Sender<Packet>,
) -> Result<()> {
    loop {
        if !processed.contains(packet.key()) {
            processed.insert(packet.key().clone());

            let message = S::Message::from_packet(0, &packet)?;

            state.apply(0, &packet)?;
            output_tx.send(packet)?;

            update(state, node, message)?;
        }
        match node_rx.try_recv() {
            Ok(next) => packet = next,
            _ => break,
        }
    }
        
    processed.clear(); 

    Ok(())
}