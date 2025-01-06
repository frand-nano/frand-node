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
    
    pub use crate::{
        bases::{
            Accessor, State, Message, Node, Emitable, Fallback, System,
            Emitter, 
        },
        extends::{
            Processor, Proxy, OptionNode, VecNode,
            OptionMessage, VecMessage,
        },
    };
}

pub mod macro_prelude {
    pub use crate::prelude::*;

    pub use crate::{
        bases::{
            Consensus,
            Result, Header, NodeKey, NodeId, 
            Callback, FutureCallback, 
            Packet, PacketError, PacketMessage, MessageError,
        },
        extends::TerminalNode,
    };
}