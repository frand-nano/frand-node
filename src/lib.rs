// Copyright 2024 frand-nano
// SPDX-License-Identifier: MIT
//
// This software is licensed under the MIT License.
// For more details, see the LICENSE file in the project root.

pub use prelude::*;

pub mod bases;
pub mod extends;

pub mod prelude {
    pub use frand_node_macro::*;
    
    pub use crate::bases::{
        State, Message, Consensus, Node, Emitter,   
    };
}

pub mod macro_prelude {
    pub use crate::prelude::*;

    pub use crate::{
        bases::{
            Result, Header, NodeKey, NodeId, 
            Callback, FutureCallback, 
            Packet, PacketError, PacketMessage, MessageError,
        },
        extends::{
            TerminalConsensus, TerminalNode,
        },
    };
}