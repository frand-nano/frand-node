use std::any::Any;
use std::fmt::Debug;
use super::*;

pub trait Message: 'static + Debug + Clone + Sized + Send + Sync {   
    fn from_state<S: State>(
        header: &Header, 
        depth: usize, 
        state: S,
    ) -> Result<Self, MessageError>;

    fn from_packet(
        packet: &Packet, 
        depth: usize, 
    ) -> Result<Self, PacketError>;

    fn to_packet(
        &self,
        header: &Header, 
    ) -> Result<Packet, MessageError>;

    unsafe fn cast_state<S1: State, S2: State>(state: S1) -> S2 {
        (&state as *const dyn Any)
        .cast::<S2>()
        .as_ref().cloned().unwrap()
    }
}