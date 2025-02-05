// Copyright 2024 frand-nano
// SPDX-License-Identifier: MIT
//
// This software is licensed under the MIT License.
// For more details, see the LICENSE file in the project root.

pub use prelude::*;

pub mod bases;
pub mod terminal;
pub mod vec;

pub mod prelude {
    pub use frand_node_macro::*;

    pub use crate::bases::{
        state::State,
        message::Message,
        consensus::Consensus,
        node::Node,
        system::{Fallback, System},
        component::Component,
    };
}

pub mod ext {
    pub use crate::{
        prelude::*,
        bases::{
            packet::{IdDelta, IdSize, AltIndex, AltSize, Key, Consist, Id, AltDepth, Alt, Payload, Packet, MessagePacket},
            callback::Callback,
            emitter::Emitter,
            accesser::{Accesser, RcAccess},
            consensus_read::ConsensusRead,
            node::NewNode,
            result::{Result, PacketError},
        },
        terminal::terminal,
        vec::vec,
    };
}

mod frand_node {
    pub mod ext {
        pub use crate::ext::*;
    }
}
