use std::{ops::DerefMut, sync::{Arc, RwLock}};
use crate::ext::*;

#[derive(Debug, Default, Clone)]
pub struct Consensus<CS: System> {
    accesser: CS::Accesser,
    emitter: CS::Emitter,
    transient: Transient,
    consensus: Arc<RwLock<CS>>,
}

impl<CS: System> Consensus<CS> {
    pub fn new(
        state: CS,
        callback: impl Fn(MessagePacket<CS>) + 'static + Send + Sync,
        process: impl Fn(MessagePacket<CS>) + 'static + Send + Sync,
    ) -> Self {
        let consensus: Arc<RwLock<CS>> = Arc::default();
        *consensus.write().unwrap() = state;

        Self { 
            accesser: Accesser::new(
                LookupBuilder { 
                    consist: Consist::default(),
                    consensus: consensus.clone(), 
                    lookup: Arc::new(|state, _| state),
                },
            ),
            emitter: Emitter::new(
                Callback::new(
                    Consist::default(), 
                    Arc::new(callback),
                    Arc::new(process),
                ),
            ), 
            transient: Transient::default(),
            consensus, 
        }
    }

    pub fn node<'c: 'n, 'n>(&'c self) -> CS::Node<'n> {
        NewNode::new(
            &self.accesser,
            &self.emitter,
            &CallbackMode::Default,
            &self.transient,
        )
    }

    pub fn process_node<'c: 'n, 'n>(&'c self) -> CS::Node<'n> {
        NewNode::new(
            &self.accesser,
            &self.emitter,
            &CallbackMode::Process,
            &self.transient,
        )
    }

    pub fn apply(&mut self, message: &CS::Message) {
        message.apply_to(self.consensus.write().unwrap().deref_mut());
    }
}