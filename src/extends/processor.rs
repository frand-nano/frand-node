use futures::{future::BoxFuture, stream::{FuturesUnordered, StreamExt}, FutureExt};
use rustc_hash::FxHasher;
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};
use std::{collections::{HashMap, HashSet, VecDeque}, hash::BuildHasherDefault, ops::Deref, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use crossbeam::channel::{unbounded, Receiver};
use bases::*;
use crate::*;

type ProcessorTask<S> = BoxFuture<'static, Result<ContextOrTask<S>>>;

pub struct Processor<A: Accessor> {
    node: A::Node,
    input_tx: UnboundedSender<PacketMessage>,
    input_rx: UnboundedReceiver<PacketMessage>,
    input_future_tx: UnboundedSender<FutureMessage>,
    input_future_rx: UnboundedReceiver<FutureMessage>,
    carry_tx: UnboundedSender<PacketMessage>,
    carry_rx: UnboundedReceiver<PacketMessage>,
    carrys: HashMap<Key, PacketMessage, BuildHasherDefault<FxHasher>>,    
    output_in_use: Arc<AtomicBool>,
    output_tx: UnboundedSender<PacketMessage>,
    output_rx: Option<UnboundedReceiver<PacketMessage>>,
    context: ProcessorContext<A>,
    contexts: Vec<ProcessorContext<A>>,
    tasks: FuturesUnordered<ProcessorTask<A>>,
    update: fn(&A::Node, A::Message, Option<f32>),
}

pub struct ProcessorContext<A: Accessor> {
    process_node: A::Node,
    processed: HashSet<Key, BuildHasherDefault<FxHasher>>,    
    process_rx: Receiver<PacketMessage>,
    process_future_rx: Receiver<FutureMessage>,
    output_in_use: Arc<AtomicBool>,
    output_tx: UnboundedSender<PacketMessage>,
    packets: VecDeque<PacketMessage>,
    futures: VecDeque<FutureMessage>,
    update: fn(&A::Node, A::Message, Option<f32>),
}

pub enum ContextOrTask<A: Accessor> {
    Context(ProcessorContext<A>),
    Task(ProcessorContext<A>),
}

impl<A: Accessor> Deref for Processor<A> {
    type Target = A::Node;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<A: Accessor> Processor<A> 
where A::Node: Consensus<A::State> {
    pub fn node(&self) -> &A::Node { &self.node }
    pub fn input_tx(&self) -> &UnboundedSender<PacketMessage> { &self.input_tx }
    pub fn input_rx(&self) -> &UnboundedReceiver<PacketMessage> { &self.input_rx }
    pub fn input_future_tx(&self) -> &UnboundedSender<FutureMessage> { &self.input_future_tx }
    pub fn input_future_rx(&self) -> &UnboundedReceiver<FutureMessage> { &self.input_future_rx }
    pub fn output_in_use(&self) -> &Arc<AtomicBool> { &self.output_in_use }
    pub fn output_tx(&self) -> &UnboundedSender<PacketMessage> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<UnboundedReceiver<PacketMessage>> { 
        self.output_in_use.store(true, Ordering::Release);
        self.output_rx.take() 
    }

    pub fn new<F>(
        callback: F,
        update: fn(&A::Node, A::Message, Option<f32>),
    ) -> Self 
    where F: 'static + Fn(Result<(), MessageError>) + Send + Sync {
        Self::new_with(A::Node::default(), callback, update)
    }

    pub fn new_with<F>(
        mut node: A::Node,
        callback: F,
        update: fn(&A::Node, A::Message, Option<f32>),
    ) -> Self 
    where F: 'static + Fn(Result<(), MessageError>) + Send + Sync {
        let (input_tx, input_rx) = unbounded_channel();
        let (input_future_tx, input_future_rx) = unbounded_channel();
        let (carry_tx, carry_rx) = unbounded_channel();
        let (output_tx, output_rx) = unbounded_channel();

        let arc_callback = Arc::new(callback);

        let arc_callback_clone = arc_callback.clone();
        let input_tx_clone = input_tx.clone();
        let callback = Callback::new(move |packet| {
            arc_callback_clone(input_tx_clone.send(packet).map_err(|err| err.into()))
        });

        let input_future_tx_clone = input_future_tx.clone();
        let future_callback = FutureCallback::new(move |future| {
            arc_callback(input_future_tx_clone.send(future).map_err(|err| err.into()))
        });

        let carry_tx_clone = carry_tx.clone();
        let carry_callback = Callback::new(move |mut packet| {
            packet.set_carry();
            carry_tx_clone.send(packet).unwrap()
        });

        let output_in_use = Arc::new(AtomicBool::new(false));

        let emitter = Emitter::new(
            callback, 
            carry_callback,
            future_callback, 
        );

        node.set_emitter(Some(emitter));

        Self {
            context: ProcessorContext::new(
                node.clone(),
                carry_tx.clone(),
                output_in_use.clone(),
                output_tx.clone(),
                update,
            ),
            node,
            input_tx, input_rx,
            input_future_tx, input_future_rx,
            carry_tx, carry_rx,
            carrys: HashMap::default(),
            output_in_use,
            output_tx, output_rx: Some(output_rx),
            contexts: Vec::new(),
            tasks: FuturesUnordered::new(),
            update,
        }
    }

    fn pop_context(&mut self) -> ProcessorContext<A> {
        self.contexts.pop().unwrap_or_else(|| {
            ProcessorContext::new(
                self.node.clone(),
                self.carry_tx.clone(),
                self.output_in_use.clone(),
                self.output_tx.clone(),
                self.update,
            ) 
        })
    }

    #[cfg(feature = "tokio_rt")]
    pub async fn start(
        self,
        tick_delta_secs: f32,
    ) -> tokio::task::JoinHandle<()> {
        use tokio::time::{self, Duration};

        let mut processor = self;
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs_f32(tick_delta_secs));
            loop { 
                if let Err(err) = processor.process_async(&mut interval).await {
                    log::error!("{err}");
                    break;
                }
            }
        })
    }

    pub fn process(&mut self) -> Result<()> {
        while let Ok(packet) = self.input_rx.try_recv() {
            self.context.packets.push_back(packet);
            self.context.process()?;
            self.context.processed.clear();
        }

        self.transfer_carry_messages()
    }

    #[cfg(feature = "tokio_rt")]
    async fn process_async(
        &mut self, 
        interval: &mut tokio::time::Interval,
    ) -> Result<()> {
        select! {
            _ = interval.tick() => {
                self.transfer_carry_messages() 
            }
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
                    ContextOrTask::Task(task) => {
                        self.tasks.push(task.process_async().boxed())
                    },
                }
                Ok(())
            }
            else => Err(Error::Text(format!(
                "input_rx channel closed"
            ))) 
        }
    }

    fn transfer_carry_messages(&mut self) -> Result<()> {        
        while let Ok(packet) = self.carry_rx.try_recv() {
            self.carrys.insert(packet.key(), packet);
        }

        for (_, packet) in self.carrys.drain() {
            self.input_tx.send(packet)?;
        }

        Ok(())
    }
}

impl<A: Accessor> ProcessorContext<A> 
where A::Node: Consensus<A::State> {
    fn new(
        node: A::Node,
        carry_tx: UnboundedSender<PacketMessage>,
        output_in_use: Arc<AtomicBool>,
        output_tx: UnboundedSender<PacketMessage>,
        update: fn(&A::Node, A::Message, Option<f32>),
    ) -> Self {
        let (process_tx, process_rx) = unbounded();
        let (process_future_tx, process_future_rx) = unbounded();

        let callback = Callback::new(move |packet| {
            process_tx.send(packet).unwrap()
        });

        let future_callback = FutureCallback::new(move |future| {
            process_future_tx.send(future).unwrap()
        });

        let carry_callback = Callback::new(move |mut packet| {
            packet.set_carry();
            carry_tx.send(packet).unwrap()
        });

        Self {
            process_node: Consensus::new_from(&node, Some(Emitter::new(
                callback, 
                carry_callback,
                future_callback, 
            ))),
            processed: HashSet::default(),
            process_rx,          
            process_future_rx,        
            output_in_use,
            output_tx,
            packets: VecDeque::new(),
            futures: VecDeque::new(),
            update,
        }
    }

    fn process(&mut self) -> Result<()> {
        Ok(while let Some(mut packet) = self.packets.pop_front() {
            loop {
                if !self.processed.contains(&packet.key()) {
                    self.processed.insert(packet.key());

                    let message = A::Message::from_packet_message(Key::default(), &packet)?;

                    self.process_node.apply(message.clone());
                    
                    let delta = packet.carry().map(|carry| {
                        carry.elapsed().as_secs_f32()
                    });

                    if self.output_in_use.load(Ordering::Acquire) {
                        self.output_tx.send(packet)?;
                    }

                    (self.update)(&self.process_node, message, delta);
                }          
                match self.process_rx.try_recv() {
                    Ok(recv) => packet = recv,
                    _ => break,
                } 
            }
        })
    }

    async fn process_async(mut self) -> Result<ContextOrTask<A>> {
        self.process()?;

        while let Ok(future) = self.process_future_rx.try_recv() {
            self.futures.push_back(future);
        }

        Ok(match self.futures.pop_front() {
            Some(future) => {
                if !self.processed.contains(&future.0.into()) {
                    self.processed.insert(future.0.into());
        
                    let packet = future.1.await;
                    let message = A::Message::from_packet_message(Key::default(), &packet)?;

                    self.process_node.apply(message.clone());
            
                    if self.output_in_use.load(Ordering::Acquire) {
                        self.output_tx.send(packet)?;
                    }

                    (self.update)(&self.process_node, message, None);
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
