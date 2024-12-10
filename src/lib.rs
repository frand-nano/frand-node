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
            State, Node, Message, StateNode, Emitter, Emitable,   
        },
        extends::Processor,        
    };
}

pub mod macro_prelude {
    pub use crate::prelude::*;

    /// This module re-exports functionality from the `anyhow` crate.
    pub mod anyhow {
        /// Re-exported from the `anyhow` crate.
        pub use anyhow::Result;
    }

    pub use crate::{
        bases::{
            Header, NodeKey, NodeId, 
            Packet, 
            Reporter,
        },
        extends::{TerminalNode, TerminalStateNode},
    };
}