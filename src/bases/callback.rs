use std::{any::type_name_of_val, future::Future, sync::Arc};
use crate::ext::*;

#[derive(Debug, Clone, Copy)]
pub enum CallbackMode {
    Default,
    Process,
}

#[derive( Clone)]
pub struct Callback<S: State>{
    consist: Consist,
    callback: Arc<dyn Fn(MessagePacket<S>) + Send + Sync>,
    process: Arc<dyn Fn(MessagePacket<S>) + Send + Sync>,
}

impl<S: State + std::fmt::Debug> std::fmt::Debug for Callback<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callback")
        .field("consist", &self.consist)
        .field("callback", &type_name_of_val(&self.callback))
        .field("process", &type_name_of_val(&self.process))
        .finish()
    }
}

impl<S: State> Callback<S> {
    pub fn consist(&self) -> &Consist { &self.consist }
    pub fn callback(&self) -> &Arc<dyn Fn(MessagePacket<S>) + Send + Sync> { &self.callback }
    pub fn process(&self) -> &Arc<dyn Fn(MessagePacket<S>) + Send + Sync> { &self.process }

    pub fn new(
        consist: Consist,
        callback: Arc<dyn Fn(MessagePacket<S>) + Send + Sync>,
        process: Arc<dyn Fn(MessagePacket<S>) + Send + Sync>,
    ) -> Self {
        Self { 
            consist, 
            callback, 
            process, 
        }
    }

    pub fn access<P: State>(
        consist: Consist,
        callback: Arc<dyn Fn(MessagePacket<P>) + Send + Sync>,
        process: Arc<dyn Fn(MessagePacket<P>) + Send + Sync>,
        id_delta: IdDelta,
        wrap: fn(AltIndex, S::Message) -> P::Message,
    ) -> Self {   
        Self { 
            consist: consist.access(id_delta, P::NODE_ALT_SIZE), 
            callback: Arc::new(move |packet| (callback)(packet.wrap(
                consist.alt_depth(), 
                wrap,
            ))), 
            process: Arc::new(move |packet| (process)(packet.wrap(
                consist.alt_depth(), 
                wrap,
            ))), 
        }     
    }

    pub fn emit(
        &self, 
        mode: &CallbackMode,
        transient: &Transient, 
        message: S::Message,
    ) {
        let message = MessagePacket::message(
            Key::new(self.consist, *transient), 
            message,
        );

        match mode {
            CallbackMode::Default => (self.callback)(message),
            CallbackMode::Process => (self.process)(message),
        }
    }

    pub fn emit_carry<F>(
        &self, 
        mode: &CallbackMode,
        transient: &Transient, 
        lookup: F,
    ) where F: Fn() -> S::Message + 'static + Send + Sync {
        let message = MessagePacket::carry(
            Key::new(self.consist, *transient), 
            lookup,
        );

        match mode {
            CallbackMode::Default => (self.callback)(message),
            CallbackMode::Process => (self.process)(message),
        }
    }

    pub fn emit_future<F>(
        &self, 
        mode: &CallbackMode,
        transient: &Transient, 
        future: F,
    ) where F: Future<Output = S::Message> + 'static + Send + Sync {
        let message = MessagePacket::future(
            Key::new(self.consist, *transient), 
            future,
        );

        match mode {
            CallbackMode::Default => (self.callback)(message),
            CallbackMode::Process => (self.process)(message),
        }
    }
}