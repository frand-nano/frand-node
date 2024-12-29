use futures::{future::BoxFuture, stream::{FuturesUnordered, StreamExt}, FutureExt};
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};
use std::{collections::{HashSet, VecDeque}, ops::Deref, sync::Arc};
use crossbeam::channel::{unbounded, Receiver};
use bases::*;
use crate::*;

type ProcessorTask<S> = BoxFuture<'static, Result<ContextOrTask<S>>>;

pub struct Processor<S: State> {
    node: S::Node<S::Message>,
    consensus: S::Consensus<S::Message>,
    input_tx: UnboundedSender<PacketMessage<S::Message>>,
    input_rx: UnboundedReceiver<PacketMessage<S::Message>>,
    input_future_tx: UnboundedSender<EmitableFuture<S::Message>>,
    input_future_rx: UnboundedReceiver<EmitableFuture<S::Message>>,
    output_tx: UnboundedSender<PacketMessage<S::Message>>,
    output_rx: Option<UnboundedReceiver<PacketMessage<S::Message>>>,
    context: ProcessorContext<S>,
    contexts: Vec<ProcessorContext<S>>,
    tasks: FuturesUnordered<ProcessorTask<S>>,
    update: fn(&S::Node<S::Message>, S::Message),
}

pub struct ProcessorContext<S: State> {
    node: S::Node<S::Message>,
    consensus: S::Consensus<S::Message>,
    processed: HashSet<NodeKey>,    
    process_rx: Receiver<PacketMessage<S::Message>>,
    process_future_rx: Receiver<EmitableFuture<S::Message>>,
    output_tx: UnboundedSender<PacketMessage<S::Message>>,
    packets: VecDeque<PacketMessage<S::Message>>,
    futures: VecDeque<EmitableFuture<S::Message>>,
    update: fn(&S::Node<S::Message>, S::Message),
}

pub enum ContextOrTask<S: State> {
    Context(ProcessorContext<S>),
    Task(ProcessorContext<S>),
}

impl<S: State> Deref for Processor<S> {
    type Target = S::Node<S::Message>;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<S: State> Processor<S> {
    pub fn node(&self) -> &S::Node<S::Message> { &self.node }
    pub fn input_tx(&self) -> &UnboundedSender<PacketMessage<S::Message>> { &self.input_tx }
    pub fn input_rx(&self) -> &UnboundedReceiver<PacketMessage<S::Message>> { &self.input_rx }
    pub fn input_future_tx(&self) -> &UnboundedSender<EmitableFuture<S::Message>> { &self.input_future_tx }
    pub fn input_future_rx(&self) -> &UnboundedReceiver<EmitableFuture<S::Message>> { &self.input_future_rx }
    pub fn output_tx(&self) -> &UnboundedSender<PacketMessage<S::Message>> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<UnboundedReceiver<PacketMessage<S::Message>>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
        update: fn(&S::Node<S::Message>, S::Message),
    ) -> Self 
    where F: 'static + Fn(Result<(), MessageError>) + Send + Sync {
        let (input_tx, input_rx) = unbounded_channel();
        let (input_future_tx, input_future_rx) = unbounded_channel();
        let (output_tx, output_rx) = unbounded_channel();

        let arc_callback = Arc::new(callback);

        let arc_callback_clone = arc_callback.clone();
        let input_tx_clone = input_tx.clone();
        let callback = move |packet| {
            arc_callback_clone(input_tx_clone.send(packet).map_err(|err| err.into()))
        };

        let input_future_tx_clone = input_future_tx.clone();
        let future_callback = move |future| {
            arc_callback(input_future_tx_clone.send(future).map_err(|err| err.into()))
        };

        let consensus = S::Consensus::default();

        Self {
            node: consensus.new_node(
                &Callback::new(callback), 
                &FutureCallback::new(future_callback),
            ),
            context: ProcessorContext::new(
                consensus.clone(),
                output_tx.clone(),
                update,
            ),
            consensus,
            input_tx, input_rx,
            input_future_tx, input_future_rx,
            output_tx,
            output_rx: Some(output_rx),
            contexts: Vec::new(),
            tasks: FuturesUnordered::new(),
            update,
        }
    }

    fn pop_context(&mut self) -> ProcessorContext<S> {
        self.contexts.pop().unwrap_or_else(|| {
            ProcessorContext::new(
                self.consensus.clone(),
                self.output_tx.clone(),
                self.update,
            ) 
        })
    }

    #[cfg(feature = "tokio_rt")]
    pub async fn start(self) -> tokio::task::JoinHandle<()> {
        let mut processor = self;
        tokio::spawn(async move {
            loop { 
                if let Err(err) = processor.process_async().await {
                    log::error!("{err}");
                    break;
                }
            }
        })
    }

    pub fn process(&mut self) -> Result<()> {
        Ok(while let Ok(packet) = self.input_rx.try_recv() {
            self.context.packets.push_back(packet);
            self.context.process()?;
            self.context.processed.clear();
        })
    }

    #[cfg(feature = "tokio_rt")]
    async fn process_async(&mut self) -> Result<()> {
        select! {
            Some(packet) = self.input_rx.recv() => {
                let mut context = self.pop_context();
                context.packets.push_back(packet);
                self.tasks.push(context.process_async().boxed());
                Ok(()) 
            }
            Some(future) = self.input_future_rx.recv() => {
                let mut context = self.pop_context();
                context.futures.push_back(future);
                self.tasks.push(context.process_async().boxed());
                Ok(()) 
            }
            Some(task) = self.tasks.next() => {
                match task? {
                    ContextOrTask::Context(context) => self.contexts.push(context),
                    ContextOrTask::Task(task) => self.tasks.push(task.process_async().boxed()),
                }
                Ok(())
            }
            else => Err(Error::Text(format!(
                "input_rx channel closed"
            ))) 
        }
    }
}

impl<S: State> ProcessorContext<S> {
    pub fn node(&self) -> &S::Node<S::Message> { &self.node }

    fn new(
        consensus: S::Consensus<S::Message>,
        output_tx: UnboundedSender<PacketMessage<S::Message>>,
        update: fn(&S::Node<S::Message>, S::Message),
    ) -> Self {
        let (process_tx, process_rx) = unbounded();
        let (process_future_tx, process_future_rx) = unbounded();

        let callback = move |packet| {
            process_tx.send(packet).unwrap()
        };

        let future_callback = move |future| {
            process_future_tx.send(future).unwrap()
        };

        Self {
            node: consensus.new_node(
                &Callback::new(callback), 
                &FutureCallback::new(future_callback),
            ),
            consensus,
            processed: HashSet::new(),
            process_rx,          
            process_future_rx,          
            output_tx,
            packets: VecDeque::new(),
            futures: VecDeque::new(),
            update,
        }
    }

    fn process(&mut self) -> Result<()> {
        Ok(while let Some(mut packet) = self.packets.pop_front() {
            loop {
                if !self.processed.contains(&packet.header) {
                    self.processed.insert(packet.header.clone());
        
                    self.consensus.apply(packet.message.clone());

                    self.output_tx.send(packet.clone())?;
        
                    (self.update)(self.node(), packet.message);
                }          
                match self.process_rx.try_recv() {
                    Ok(next) => packet = next,
                    _ => break,
                }  
            }
        })
    }

    async fn process_async(mut self) -> Result<ContextOrTask<S>> {
        self.process()?;

        while let Ok(future) = self.process_future_rx.try_recv() {
            self.futures.push_back(future);
        }

        Ok(match self.futures.pop_front() {
            Some(future) => {
                if !self.processed.contains(&future.0) {
                    self.processed.insert(future.0.clone());
        
                    let message = future.1.await;
                    self.consensus.apply(message.clone());
    
                    self.output_tx.send(PacketMessage {
                        header: future.0,
                        message: message.clone(),
                    })?;
        
                    (self.update)(self.node(), message);
                }

                let packet = self.process_rx.try_recv();
                let future = self.process_future_rx.try_recv();

                match (&packet, &future) {
                    (Err(_), Err(_)) => {
                        self.processed.clear();
                        ContextOrTask::Context(self)
                    },
                    _ => {
                        if let Ok(packet) = packet {
                            self.packets.push_back(packet);
                            while let Ok(packet) = self.process_rx.try_recv() {
                                self.packets.push_back(packet);
                            }
                        }

                        if let Ok(future) = future {
                            self.futures.push_back(future);
                            while let Ok(future) = self.process_future_rx.try_recv() {
                                self.futures.push_back(future);
                            }
                        }

                        ContextOrTask::Task(self)
                    },
                }
            },
            None => {
                self.processed.clear();
                ContextOrTask::Context(self)
            }, 
        })
    }
}
