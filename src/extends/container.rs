use std::ops::Deref;
use crossbeam::channel::{unbounded, Receiver, SendError, Sender};
use bases::*;
use crate::*;

pub struct Container<S: State> {
    node: S::Node<S::Message>,
    consensus: S::Consensus<S::Message>,
    input_tx: Sender<PacketMessage<S::Message>>,
    input_rx: Receiver<PacketMessage<S::Message>>,
    output_tx: Sender<PacketMessage<S::Message>>,
    output_rx: Option<Receiver<PacketMessage<S::Message>>>,
}

impl<S: State> Deref for Container<S> {
    type Target = S::Node<S::Message>;
    fn deref(&self) -> &Self::Target { &self.node }
}

impl<S: State> Container<S> {
    pub fn node(&self) -> &S::Node<S::Message> { &self.node }
    pub fn input_tx(&self) -> &Sender<PacketMessage<S::Message>> { &self.input_tx }
    pub fn output_tx(&self) -> &Sender<PacketMessage<S::Message>> { &self.output_tx }
    pub fn take_output_rx(&mut self) -> Option<Receiver<PacketMessage<S::Message>>> { self.output_rx.take() }

    pub fn new<F>(
        callback: F,
    ) -> Self where F: 'static + Fn(Result<(), SendError<PacketMessage<S::Message>>>) + Send + Sync {
        let (input_tx, input_rx) = unbounded();
        let (output_tx, output_rx) = unbounded();

        let input_tx_clone = input_tx.clone();
        let callback = move |message| {
            callback(input_tx_clone.send(message))
        };

        let consensus = S::Consensus::default();

        Self {
            node: consensus.new_node(
                &Callback::new(callback), 
                &FutureCallback::default(),
            ),
            consensus,
            input_tx, input_rx,
            output_tx,
            output_rx: Some(output_rx),
        }
    }

    pub fn process(&mut self) -> Result<()> {
        Ok(while let Ok(packet) = self.input_rx.try_recv() {    
            self.consensus.apply(packet.message.clone());

            if self.output_rx.is_none() {
                self.output_tx.send(packet)?; 
            }
        })        
    }
}