use anyhow::{anyhow, Result};
use tokio::{runtime::Handle, select, sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender}, task::JoinHandle};
use std::{collections::HashSet, ops::Deref};
use bases::{AnchorKey, EmitableFuture, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver};
use crate::*;

pub struct AsyncProcessor<S: State> {
    state: S,
    input_anchor: S::Anchor,
    process_anchor: S::Anchor,
    processed: HashSet<AnchorKey>,    
    input_tx: UnboundedSender<EmitableFuture>,
    input_rx: UnboundedReceiver<EmitableFuture>,
    process_rx: Receiver<EmitableFuture>,
    message_tx: UnboundedSender<S::Message>,
    message_rx: UnboundedReceiver<S::Message>,
    output_tx: UnboundedSender<Packet>,
    output_rx: Option<UnboundedReceiver<Packet>>,
    update: fn(&S::Node<'_>, S::Message),
}

impl<S: State> Deref for AsyncProcessor<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target { &self.state }
}

impl<S: State> AsyncProcessor<S> {
    pub fn input_tx(&self) -> &UnboundedSender<EmitableFuture> { &self.input_tx }
    pub fn input_rx(&self) -> &UnboundedReceiver<EmitableFuture> { &self.input_rx }
    pub fn state(&self) -> &S { &self.state }
    pub fn anchor(&self) -> &S::Anchor { &self.input_anchor }
    pub fn new_node(&self) -> S::Node<'_> { self.state.with(&self.input_anchor) }
    pub fn output_tx(&self) -> &UnboundedSender<Packet> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<UnboundedReceiver<Packet>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
        update: fn(&S::Node<'_>, S::Message),
    ) -> Self where F: 'static + Fn(Result<(), SendError<EmitableFuture>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded_channel();
        let (process_tx, process_rx) = unbounded();
        let (message_tx, message_rx) = unbounded_channel();
        let (output_tx, output_rx) = unbounded_channel();

        let input_tx_clone = input_tx.clone();
        let input_callback = move |future| {
            callback(input_tx_clone.send(future))
        };

        let process_callback = move |future| {
            process_tx.send(future).unwrap()
        };

        Self {
            state: S::default(),
            input_anchor: S::new_anchor(Reporter::new_future_callback(input_callback)),
            process_anchor: S::new_anchor(Reporter::new_future_callback(process_callback)),
            processed: HashSet::new(),
            input_tx, input_rx,
            process_rx,
            message_tx, message_rx,            
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

    pub async fn process<'n>(&'n mut self) -> Result<()> {
        select! {
            Some(mut future) = self.input_rx.recv() => {
                let mut node = self.state.with(&self.process_anchor);

                loop {
                    if !self.processed.contains(&future.0) {
                        self.processed.insert(future.0.clone());
            
                        let packet = future.1.await.to_packet(&future.0);
                        let message = node.apply_export(0, &packet)?;
    
                        if self.output_rx.is_none() {
                            self.output_tx.send(packet)?;
                        }
    
                        self.message_tx.send(message.clone())
                        .map_err(|err| anyhow!("{err}"))?;
            
                        (self.update)(&node, message);
                    }
                    match self.process_rx.try_recv() {
                        Ok(next) => future = next,
                        _ => break,
                    }
                }
                    
                self.processed.clear();
            }
            Some(message) = self.message_rx.recv() => {
                self.state.apply_message(message);
            }
        }
    
        Ok(())
    }
}
