use crate::{
    lib::{
        fmt::{self, Display, Formatter},
        Vec,
    },
    Address, ProtocolId, Route,
};
use serde::{Deserialize, Serialize};

/// A generic transport message
///
/// While this type is exposed in ockam_core (and the root `ockam`
/// crate) in order to provide a mechanism for third-party developers
/// to create custom transport channel routers.  Casual users of ockam
/// should never have to interact with this type directly.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TransportMessage {
    /// The transport protocol version
    pub version: u8,
    /// Onward message route
    pub onward_route: Route,
    /// Return message route
    ///
    /// This field must be populated by routers handling this message
    /// along the way.
    pub return_route: Route,
    /// A protocol identifier for this message
    pub protocol: ProtocolId,
    /// The message payload
    pub payload: Vec<u8>,
}

/// Example protocol payload
pub struct ProtocolPayload {
    /// Protocol ID
    pub proto: ProtocolId,
    /// Whatever data
    pub data: Vec<u8>,
}

impl TransportMessage {
    /// Create a new v1 transport message with empty return route
    pub fn v1(onward_route: Route, payload: Vec<u8>) -> Self {
        Self::v1_protocol(onward_route, payload, ProtocolId::none())
    }

    /// Create a new v1 transport message for a particular protocol
    pub fn v1_protocol<P: Into<ProtocolId>>(
        onward_route: Route,
        payload: Vec<u8>,
        protocol: P,
    ) -> Self {
        Self {
            version: 1,
            onward_route,
            return_route: Route::new().into(),
            protocol: protocol.into(),
            payload,
        }
    }
}

impl Display for TransportMessage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Message (onward route: {}, return route: {})",
            self.onward_route, self.return_route
        )
    }
}

/// A command message for router implementations
///
/// If a router is implemented as a worker, it should accept this
/// message type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum RouterMessage {
    /// Route the provided message towards its destination
    Route(TransportMessage),
    /// Register a new client to this routing scope
    Register {
        /// Specify an accept scope for this client
        accepts: Address,
        /// The clients own worker bus address
        self_addr: Address,
    },
}
