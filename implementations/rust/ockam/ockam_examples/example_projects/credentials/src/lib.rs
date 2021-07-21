use ockam::{EntityIdentifier, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::stdin;

pub const OFFICE_TCP_ADDRESS: &str = "127.0.0.1:4222";
pub const OFFICE_LISTENER_ADDRESS: &str = "office_listener";
pub const OFFICE_ISSUER_ADDRESS: &str = "office_issuer";
pub const DOOR_TCP_ADDRESS: &str = "127.0.0.1:5333";
pub const DOOR_LISTENER_ADDRESS: &str = "door_listener";
pub const DOOR_VERIFIER_ADDRESS: &str = "door_verifier";
pub const DOOR_CONTROLLER_ADDRESS: &str = "door_controller";

#[derive(Serialize, Deserialize)]
pub struct OpenDoorMessage;

#[derive(Serialize, Deserialize)]
pub struct DoorIsOpenedMessage;

pub fn read_entity_id() -> Result<EntityIdentifier> {
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();
    let line = line.replace(&['\n', '\r'][..], "");

    EntityIdentifier::try_from(line.as_str())
}
