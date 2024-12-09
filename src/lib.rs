pub use prelude::*;

pub mod bases;
pub mod extends;

pub mod prelude {
    pub use frand_node_macro::*;
    
    pub use crate::bases::{
        State, NodeBase, Message, Emitter,    
        Processor,
    };
}

pub mod macro_prelude {
    pub use crate::prelude::*;

    pub mod anyhow {
        pub use anyhow::Result;
    }

    pub use crate::{
        bases::{
            Header, NodeKey, NodeId, 
            Packet, 
            Reporter,
        },
        extends::TerminalNode,
    };
}