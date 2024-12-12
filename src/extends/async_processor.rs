use anyhow::{anyhow, bail, Result};
use futures::{future::BoxFuture, stream::{FuturesUnordered, StreamExt}, FutureExt};
use tokio::{select, spawn, sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender}, task::JoinHandle};
use std::{collections::{HashSet, VecDeque}, ops::Deref};
use bases::{NodeKey, EmitableFuture, Packet, Reporter};
use crossbeam::channel::{unbounded, Receiver};
use crate::*;

type AsyncProcessorTask<S> = BoxFuture<'static, Result<ContextOrTask<S>>>;

pub struct AsyncProcessor<S: State> {
    node: S::Node,
    input_tx: UnboundedSender<EmitableFuture>,
    input_rx: UnboundedReceiver<EmitableFuture>,
    output_tx: UnboundedSender<Packet>,
    output_rx: Option<UnboundedReceiver<Packet>>,
    contexts: Vec<AsyncProcessorContext<S>>,
    tasks: FuturesUnordered<AsyncProcessorTask<S>>,
    update: fn(&S::Node, S::Message),
}

pub struct AsyncProcessorContext<S: State> {
    process_node: S::Node,
    processed: HashSet<NodeKey>,    
    process_rx: Receiver<EmitableFuture>,
    output_tx: UnboundedSender<Packet>,
    futures: VecDeque<EmitableFuture>,
    update: fn(&S::Node, S::Message),
}

pub enum ContextOrTask<S: State> {
    Context(AsyncProcessorContext<S>),
    Task(AsyncProcessorContext<S>),
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
        let (output_tx, output_rx) = unbounded_channel();

        let input_tx_clone = input_tx.clone();
        let input_callback = move |future| {
            callback(input_tx_clone.send(future))
        };

        let node = S::new_node(Reporter::new_future_callback(input_callback));

        Self {
            node,
            input_tx, input_rx,
            output_tx,
            output_rx: Some(output_rx),
            contexts: Vec::new(),
            tasks: FuturesUnordered::new(),
            update,
        }
    }

    fn pop_context(&mut self) -> AsyncProcessorContext<S> {
        self.contexts.pop().unwrap_or_else(|| {
            AsyncProcessorContext::new(
                &self.node,
                self.output_tx.clone(),
                self.update,
            )
        })
    }

    pub async fn start(self) -> JoinHandle<()> {
        let mut processor = self;
        spawn(async move {
            loop { 
                if let Err(err) = processor.process().await {
                    log::error!("{err}");
                    break;
                }
            }
        })
    }

    pub async fn process(&mut self) -> Result<()> {
        select! {
            Some(future) = self.input_rx.recv() => {
                let mut context = self.pop_context();
                context.futures.push_back(future);
                self.tasks.push(context.process().boxed());
                Ok(()) 
            }
            Some(task) = self.tasks.next() => {
                match task? {
                    ContextOrTask::Context(context) => self.contexts.push(context),
                    ContextOrTask::Task(task) => self.tasks.push(task.process().boxed()),
                }
                Ok(())
            }
            else => Err(anyhow!("input_rx channel closed")) 
        }
    }
}

impl<S: State> AsyncProcessorContext<S> {
    pub fn new(
        node: &S::Node,
        output_tx: UnboundedSender<Packet>,
        update: fn(&S::Node, S::Message),
    ) -> Self {
        let (process_tx, process_rx) = unbounded();

        let process_callback = move |future| {
            process_tx.send(future).unwrap()
        };

        Self {
            process_node: S::new_node_from(&node, Reporter::new_future_callback(process_callback)),
            processed: HashSet::new(),
            process_rx,          
            output_tx,
            futures: VecDeque::new(),
            update,
        }
    }

    pub async fn process(mut self) -> Result<ContextOrTask<S>> {
        match self.futures.pop_front() {
            Some(future) => {
                if !self.processed.contains(&future.0) {
                    self.processed.insert(future.0.clone());
        
                    let packet = future.1.await.to_packet(&future.0);
                    let message = self.process_node.apply_export(0, &packet)?;
    
                    self.output_tx.send(packet)?;
        
                    (self.update)(&self.process_node, message);
                }

                match self.process_rx.try_recv() {
                    Ok(future) => {
                        self.futures.push_back(future);
                        while let Ok(future) = self.process_rx.try_recv() {
                            self.futures.push_back(future);
                        }
                        Ok(ContextOrTask::Task(self))
                    },
                    Err(_) => {
                        self.processed.clear();
                        Ok(ContextOrTask::Context(self))
                    },
                }
            },
            None => bail!("AsyncProcessorContext::process() called but has no future for process"),
        }
    }
}
