use std::{any::type_name_of_val, fmt::Debug, ops::Deref, sync::Arc};
use crate::ext::*;

pub trait Accesser<S: State, CS: System>: Debug + Clone + Send + Sync + Deref<Target = RcAccess<S, CS>> {
    fn new(
        state: RcAccess<S, CS>,
    ) -> Self;
}

#[derive(Clone)]
pub struct RcAccess<S: State, CS: System> {
    consist: Consist,
    access: Arc<dyn Fn(&CS, Transient) -> &S + Send + Sync>,
}

impl<S: State + Debug, CS: System + Debug> Debug for RcAccess<S, CS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcAccess")
        .field("consist", &self.consist)
        .field("access", &type_name_of_val(&self.access))
        .finish()
    }
}

impl<S: State, CS: System> Deref for RcAccess<S, CS> {
    type Target = Arc<dyn Fn(&CS, Transient) -> &S + Send + Sync>;
    fn deref(&self) -> &Self::Target { &self.access }
}

impl<S: State, CS: System> RcAccess<S, CS> {
    pub fn new(
        consist: Consist,
        access: Arc<dyn Fn(&CS, Transient) -> &S + Send + Sync>,
    ) -> Self {
        Self { 
            consist, 
            access, 
        }
    }

    pub fn access<P: System>(
        parent: RcAccess<P, CS>,
        id_delta: IdDelta,
        alt_size: AltSize,
        access: fn(&P, AltIndex) -> &S,
    ) -> Self {   
        Self { 
            consist: parent.consist.access(id_delta, alt_size), 
            access: Arc::new(
                move |consensus, transient| access(
                    parent(consensus, transient), 
                    transient.index(parent.consist.alt_depth()),
                )
            ), 
        }     
    }
}