use anyhow::Result;
use tokio::{runtime::Handle, sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender}, task::JoinHandle};
use std::{collections::HashSet, ops::Deref};
use bases::{NodeKey, EmitableFuture, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver};
use crate::*;

pub struct AsyncProcessor<S: State> {
    node: S::Node,
    process_node: S::Node,
    processed: HashSet<NodeKey>,    
    input_tx: UnboundedSender<EmitableFuture>,
    input_rx: UnboundedReceiver<EmitableFuture>,
    process_rx: Receiver<EmitableFuture>,
    output_tx: UnboundedSender<Packet>,
    output_rx: Option<UnboundedReceiver<Packet>>,
    update: fn(&S::Node, S::Message),
}

impl<S: State> Deref for AsyncProcessor<S> {
    type Target = S::Node;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<S: State> AsyncProcessor<S> {
    pub fn node(&self) -> &S::Node { &self.node }
    pub fn input_tx(&self) -> &UnboundedSender<EmitableFuture> { &self.input_tx }
    pub fn input_rx(&self) -> &UnboundedReceiver<EmitableFuture> { &self.input_rx }
    pub fn output_tx(&self) -> &UnboundedSender<Packet> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<UnboundedReceiver<Packet>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
        update: fn(&S::Node, S::Message),
    ) -> Self where F: 'static + Fn(Result<(), SendError<EmitableFuture>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded_channel();
        let (process_tx, process_rx) = unbounded();
        let (output_tx, output_rx) = unbounded_channel();

        let input_tx_clone = input_tx.clone();
        let input_callback = move |future| {
            callback(input_tx_clone.send(future))
        };

        let process_callback = move |future| {
            process_tx.send(future).unwrap()
        };

        let node = S::new_node(Reporter::new_future_callback(input_callback));

        Self {
            process_node: S::new_node_from(&node, Reporter::new_future_callback(process_callback)),
            node,
            processed: HashSet::new(),
            input_tx, input_rx,
            process_rx,          
            output_tx,
            output_rx: Some(output_rx),
            update,
        }
    }

    pub async fn start(mut self) -> JoinHandle<()> {
        Handle::current().spawn(async move {
            loop { self.process().await.unwrap() }
        })
    }

    pub async fn process(&mut self) -> Result<()> {
        Ok(if let Some(mut future) = self.input_rx.recv().await {
            loop {
                if !self.processed.contains(&future.0) {
                    self.processed.insert(future.0.clone());
        
                    let packet = future.1.await.to_packet(&future.0);
                    let message = self.node.apply_export(0, &packet)?;

                    if self.output_rx.is_none() {
                        self.output_tx.send(packet)?;
                    }
        
                    (self.update)(&self.process_node, message);
                }
                match self.process_rx.try_recv() {
                    Ok(next) => future = next,
                    _ => break,
                }
            }
                
            self.processed.clear();
        })
    }
}
