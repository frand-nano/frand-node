// Copyright 2024 frand-nano
// SPDX-License-Identifier: MIT
//
// This software is licensed under the MIT License.
// For more details, see the LICENSE file in the project root.

pub use prelude::*;

pub mod bases;

pub mod terminal;
pub mod vec;
pub mod proxy;

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
            packet::{IdDelta, IdSize, AltIndex, AltSize, Key, Consist, Id, AltDepth, Transient, Payload, Packet, MessagePacket},
            callback::{Callback, CallbackMode},
            lookup::{Lookup, LookupBuilder},
            emitter::Emitter,
            accesser::Accesser,
            node::{NewNode, NodeAlt},
            result::{Result, PacketError},
        },
        terminal::terminal,
        vec::vec,
        proxy::{proxy, Proxy},
    };
}

mod frand_node {
    pub mod ext {
        pub use crate::ext::*;
    }
}
