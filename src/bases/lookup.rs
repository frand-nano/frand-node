use std::{any::type_name_of_val, sync::{Arc, RwLock}};
use crate::ext::*;

#[derive(Clone)]
pub struct Lookup<T: 'static> {
    lookup: Arc<dyn Fn(&Transient) -> Option<T> + Send + Sync>,
}

#[derive(Clone)]
pub struct LookupBuilder<CS: System, P: State> {
    pub consist: Consist,
    pub consensus: Arc<RwLock<CS>>,
    pub lookup: Arc<dyn Fn(Option<&CS>, Transient) -> Option<&P> + Send + Sync>,
}

impl<T: 'static + std::fmt::Debug> std::fmt::Debug for Lookup<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lookup")
        .field("lookup", &type_name_of_val(&self.lookup))
        .finish()
    }
}

impl<T: 'static> Lookup<T> {
    pub fn get(&self, transient: &Transient) -> Option<T> { 
        (self.lookup)(transient) 
    }
}

impl<CS: System, P: State> LookupBuilder<CS, P> {
    pub fn access<S: State>(
        &self,
        access: fn(Option<&P>, AltIndex) -> Option<&S>,
        id_delta: IdDelta,
    ) -> LookupBuilder<CS, S> {   
        let Self { consist, consensus, lookup } = self.clone();

        LookupBuilder { 
            consist: consist.access(id_delta, P::NODE_ALT_SIZE), 
            lookup: Arc::new(move |consensus, transient| {
                access(
                    lookup(consensus, transient), 
                    transient.index(consist.alt_depth()),
                )
            }), 
            consensus,
        }     
    }

    pub fn build<T: 'static>(
        self,
        access: fn(Option<&P>) -> Option<T>,
    ) -> Lookup<T> {   
        let consensus = self.consensus;
        let lookup = self.lookup;

        Lookup { 
            lookup: Arc::new(move |transient| {
                access(lookup(Some(&consensus.read().unwrap()), *transient))
            }), 
        }     
    }
}