use std::{any::type_name_of_val, future::Future, ops::Deref, sync::Arc, time::Instant};
use crate::ext::*;

#[derive(Clone)]
pub struct Callback<M: 'static>{
    consist: Consist,
    callback: Arc<dyn Fn(MessagePacket<M>) + Send + Sync>,
}

impl<M: 'static + std::fmt::Debug> std::fmt::Debug for Callback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callback")
        .field("consist", &self.consist)
        .field("callback", &type_name_of_val(&self.callback))
        .finish()
    }
}

impl<M: 'static> Deref for Callback<M> {
    type Target = Arc<dyn Fn(MessagePacket<M>) + Send + Sync>;
    fn deref(&self) -> &Self::Target { &self.callback }
}

impl<M: 'static> Callback<M> {
    pub fn consist(&self) -> &Consist { &self.consist }

    pub fn new(
        consist: Consist,
        callback: Arc<dyn Fn(MessagePacket<M>) + Send + Sync>,
    ) -> Self {
        Self { 
            consist, 
            callback, 
        }
    }

    pub fn access<P: 'static>(
        callback: Callback<P>,
        id_delta: IdDelta,
        alt_size: AltSize,
        wrap: fn(AltIndex, M) -> P,
    ) -> Self {   
        Self { 
            consist: callback.consist.access(id_delta, alt_size), 
            callback: Arc::new(move |packet| callback(packet.wrap(
                callback.consist.alt_depth(), 
                wrap,
            ))), 
        }     
    }

    pub fn emit(
        &self, 
        alt: &Alt, 
        message: M,
    ) {
        self(MessagePacket::Message(
            Key::new(self.consist, *alt), 
            None, 
            message,
        ));
    }

    pub fn emit_carry(
        &self, 
        alt: &Alt, 
        message: M,
    ) {
        self(MessagePacket::Carry(
            Key::new(self.consist, *alt), 
            Instant::now(), 
            message
        ));
    }

    pub fn emit_future<F>(
        &self, 
        alt: &Alt, 
        future: F,
    ) where F: Future<Output = M> + 'static + Send + Sync {
        self(MessagePacket::new_future(
            Key::new(self.consist, *alt), 
            future
        ));
    }
}