use std::{collections::{HashMap, HashSet}, hash::BuildHasherDefault, ops::Deref, task::{Context, Poll}};
use futures::{stream::{FuturesUnordered, StreamExt}, task::noop_waker_ref, FutureExt};
use rustc_hash::FxHasher;
use smallvec::SmallVec;
use tokio::{select, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};
use crate::ext::*;
use super::packet::{MessagePacketCarry, MessagePacketFuture, MessagePacketMessage};

type Input<M> = SmallVec<[MessagePacket<M>; 4]>;
type Output<M> = SmallVec<[MessagePacketMessage<M>; 8]>;

#[derive(Debug)]
pub struct Component<S: System> {
    consensus: Consensus<S>,
    input_tx: UnboundedSender<MessagePacket<S>>,
    input_rx: UnboundedReceiver<MessagePacket<S>>,
    process_rx: UnboundedReceiver<MessagePacket<S>>,
    carry: HashMap<Key, MessagePacketCarry<S>, BuildHasherDefault<FxHasher>>,   
    future: FuturesUnordered<MessagePacketFuture<S>>,
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
    pub fn consensus(&self) -> &Consensus<S> { &self.consensus }

    pub fn new(state: S) -> Self {
        let (input_tx, input_rx) = unbounded_channel();
        let (process_tx, process_rx) = unbounded_channel();

        let input_tx_clone = input_tx.clone();
        let process_tx_clone = process_tx.clone();
        let consensus: Consensus<S> = Consensus::new(
            state,
            move |message| input_tx_clone.send(message).unwrap(),
            move |message| process_tx_clone.send(message).unwrap(),
        );

        Self { 
            consensus, 
            input_tx, input_rx,
            process_rx,
            carry: HashMap::default(),
            future: FuturesUnordered::new(),
            updated: HashSet::default(),
        }        
    }

    pub fn try_update(&mut self) -> Output<S> {
        let context = &mut Context::from_waker(noop_waker_ref());
        let mut input: Input<S> = SmallVec::new();

        while let Ok(packet) = self.input_rx.try_recv() {
            input.push(packet);
        }

        while let Poll::Ready(Some(message)) = self.future.next().poll_unpin(context) {
            input.push(MessagePacket::Message(message));
        }

        self.process(input)
    }

    pub async fn update(&mut self) -> Output<S> {   
        let mut input: Input<S> = SmallVec::new();

        select! {            
            Some(packet) = self.input_rx.recv() => {
                input.push(packet);
                while let Ok(packet) = self.input_rx.try_recv() {
                    input.push(packet);
                }
            }
            Some(packet) = self.future.next() => {
                let context = &mut Context::from_waker(noop_waker_ref());

                input.push(MessagePacket::Message(packet));

                while let Poll::Ready(Some(packet)) = self.future.next().poll_unpin(context) {
                    input.push(MessagePacket::Message(packet));
                }
            }
            else => {}
        }

        self.process(input)
    }

    fn process(&mut self, input: Input<S>) -> Output<S> {
        let mut output: Output<S> = SmallVec::new();
        
        for packet in input {
            let mut packet = Some(packet);

            loop {
                match packet.take() {
                    Some(MessagePacket::Message(packet)) => {
                        if !self.updated.contains(&packet.key) {
                            self.updated.insert(packet.key);
        
                            let message = packet.message.clone();

                            self.consensus.apply(&message);
        
                            let delta = packet.instant.map(|instant| instant.elapsed());

                            S::handle(
                                self.process_node(), 
                                message, 
                                delta,
                            );
        
                            output.push(packet);
                        }
                    },
                    Some(MessagePacket::Carry(packet)) => {
                        if !self.updated.contains(&packet.key) {
                            self.updated.insert(packet.key);
        
                            let message = (packet.lookup)();

                            self.consensus.apply(&message);
        
                            let delta = Some(packet.instant.elapsed());

                            S::handle(
                                self.process_node(), 
                                message.clone(), 
                                delta,
                            );
        
                            output.push(MessagePacketMessage {
                                key: packet.key,
                                instant: Some(packet.instant),
                                message,
                            });
                        }
                    },
                    Some(MessagePacket::Future(_)) => {
                        unreachable!();
                    },
                    None => break,
                }                      

                while let Ok(recv) = self.process_rx.try_recv() {
                    match recv {
                        MessagePacket::Message(recv) => {
                            packet = Some(MessagePacket::Message(recv));
                            break;
                        },
                        MessagePacket::Carry(recv) => {
                            self.carry.insert(recv.key, recv);
                        },
                        MessagePacket::Future(recv) => {
                            self.future.push(recv);
                        },
                    }
                }
            }

            self.updated.clear();
        }

        for (_, carry) in self.carry.drain() {
            self.input_tx.send(MessagePacket::Carry(carry)).unwrap();
        }

        output
    }
} 