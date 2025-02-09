use std::{ops::DerefMut, sync::{Arc, RwLock}};
use crate::ext::*;

#[derive(Debug, Default, Clone)]
pub struct Consensus<CS: System> {
    emitter: CS::Emitter,
    accesser: CS::Accesser<CS>,
    state: Arc<RwLock<CS>>,
}

impl<CS: System> Consensus<CS> {
    pub fn new(
        callback: impl Fn(MessagePacket<CS::Message>) + 'static + Send + Sync,
    ) -> Self {
        Self { 
            emitter: Emitter::new(
                Callback::new(Consist::default(), Arc::new(callback)),
            ), 
            accesser: Accesser::new(
                RcAccess::new(Consist::default(), Arc::new(|state, _| state)),
            ),
            state: Arc::default(), 
        }
    }

    pub fn read<'c: 'n, 'n>(&'c self) -> ConsensusRead<'n, CS, CS> {
        ConsensusRead::new(
            &self.emitter,
            &self.accesser,
            Arc::new(self.state.as_ref().read().unwrap()),
            Transient::default(), 
        )
    }

    pub fn read_with<'c: 'n, 'n>(&'c self, emitter: &'c CS::Emitter) -> ConsensusRead<'n, CS, CS> {
        ConsensusRead::new(
            emitter,
            &self.accesser,
            Arc::new(self.state.as_ref().read().unwrap()),
            Transient::default(), 
        )
    }

    pub fn apply(&mut self, message: &CS::Message) {
        message.apply_to(self.state.write().unwrap().deref_mut());
    }
}