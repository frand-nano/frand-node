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
            State, Anchor, Message, Node, Emitter, Emitable,   
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
            Header, AnchorKey, AnchorId, 
            Packet, 
            Reporter,
        },
        extends::{TerminalAnchor, TerminalNode},
    };
}