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
        callback: impl Fn(MessagePacket<CS::Message>) + 'static + Send + Sync,
    ) -> Self {
        let consensus: Arc<RwLock<CS>> = Arc::default();

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
            &self.transient,
        )
    }

    pub fn node_with<'c: 'n, 'n>(
        &'c self, 
        emitter: &'c CS::Emitter,
    ) -> CS::Node<'n> {
        NewNode::new(
            &self.accesser,
            emitter,
            &self.transient,
        )
    }

    pub fn apply(&mut self, message: &CS::Message) {
        message.apply_to(self.consensus.write().unwrap().deref_mut());
    }
}