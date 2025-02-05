use std::fmt::Debug;
use crate::ext::*;

pub trait Message: 'static + Debug + Clone + Sized + Send + Sync + Unpin {
    type State: State<Message = Self>;

    fn from_packet(
        packet: &Packet,
        parent_key: Key,
        depth: usize,
    ) -> Result<Self>;
    
    fn to_packet(
        &self, 
        key: Key,
    ) -> Packet;

    fn apply_to(&self, state: &mut Self::State);   
}