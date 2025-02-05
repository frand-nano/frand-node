use std::{collections::HashSet, hash::BuildHasherDefault, ops::Deref, sync::Arc, task::{Context, Poll}};
use futures::{stream::{FuturesUnordered, StreamExt}, task::noop_waker_ref, FutureExt};
use rustc_hash::FxHasher;
use smallvec::SmallVec;
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};
use crate::ext::*;

type Input<M> = SmallVec<[MessagePacket<M>; 4]>;
type Output<M> = SmallVec<[MessagePacket<M>; 8]>;

#[derive(Debug)]
pub struct Component<S: System> {
    consensus: Consensus<S>,
    input_tx: UnboundedSender<MessagePacket<S::Message>>,
    input_rx: UnboundedReceiver<MessagePacket<S::Message>>,
    process_emitter: S::Emitter,
    process_rx: UnboundedReceiver<MessagePacket<S::Message>>,
    future: FuturesUnordered<MessagePacket<S::Message>>,
    updated: HashSet<Key, BuildHasherDefault<FxHasher>>,   
}

impl<S: System + Default> Default for Component<S> {
    fn default() -> Self { Self::new(S::default()) }
}

impl<S: System> Deref for Component<S> {
    type Target = Consensus<S>;
    fn deref(&self) -> &Self::Target { &self.consensus }
}

impl<S: System> Component<S> {
    pub fn consensus(&self) -> &Consensus<S> 
    where Consensus<S>: Send + Sync { &self.consensus }

    pub fn read<'c: 'n, 'n>(&'c self) -> ConsensusRead<'n, S, S> { self.consensus.read() }

    pub fn new(state: S) -> Self {
        let (input_tx, input_rx) = unbounded_channel();
        let (process_tx, process_rx) = unbounded_channel();

        let input_tx_clone = input_tx.clone();
        let consensus: Consensus<S> = Consensus::new(
            move |message| input_tx_clone.send(message).unwrap(),
        );

        let process_emitter = Emitter::new(
            Callback::new(
                Consist::default(), 
                Arc::new(move |message| process_tx.send(message).unwrap()),
            ),
        );

        consensus.read().node().emit(state);

        Self { 
            consensus, 
            input_tx, input_rx,
            process_emitter, 
            process_rx,
            future: FuturesUnordered::new(),
            updated: HashSet::default(),
        }        
    }

    pub fn try_update(&mut self) -> Output<S::Message> {
        let context = &mut Context::from_waker(noop_waker_ref());
        let mut input: Input<S::Message> = SmallVec::new();

        while let Ok(packet) = self.input_rx.try_recv() {
            input.push(packet);
        }

        while let Poll::Ready(Some(packet)) = self.future.next().poll_unpin(context) {
            input.push(packet);
        }

        self.process(input)
    }

    pub async fn update(&mut self) -> Output<S::Message> {   
        let mut input: Input<S::Message> = SmallVec::new();

        select! {            
            Some(packet) = self.input_rx.recv() => {
                input.push(packet);
                while let Ok(packet) = self.input_rx.try_recv() {
                    input.push(packet);
                }
            }
            Some(packet) = self.future.next() => {
                input.push(packet);
                let context = &mut Context::from_waker(noop_waker_ref());
                while let Poll::Ready(Some(packet)) = self.future.next().poll_unpin(context) {
                    input.push(packet);
                }
            }
            else => {}
        }

        self.process(input)
    }

    fn process(&mut self, input: Input<S::Message>) -> Output<S::Message> {
        let mut output: Output<S::Message> = SmallVec::new();
        
        for packet in input {
            let mut packet = packet;
            loop {
                match &packet {
                    MessagePacket::Message(key, instant, message) => {
                        if !self.updated.contains(key) {
                            self.updated.insert(*key);
        
                            self.consensus.apply(message);
        
                            let delta = instant.map(|instant| instant.elapsed());

                            S::handle(
                                self.consensus.read_with(&self.process_emitter).node(), 
                                message, 
                                delta,
                            );
        
                            output.push(packet);
                        }
                    },
                    MessagePacket::Carry(key, instant, message) => {
                        self.input_tx.send(
                            MessagePacket::Message(*key, Some(*instant), message.clone())
                        ).unwrap();
                    },
                    MessagePacket::Future(_, _, _) => {
                        self.future.push(packet);
                    },
                }                      
                match self.process_rx.try_recv() {
                    Ok(recv) => packet = recv,
                    _ => break,
                }
            }
            self.updated.clear();
        }

        output
    }
} 