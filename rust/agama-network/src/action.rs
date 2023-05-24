use crate::model::{Connection, DeviceType};
use uuid::Uuid;

/// Networking actions, like adding, updating or removing connections.
///
/// These actions are meant to be processed by [crate::system::NetworkSystem], updating the model
/// and the D-Bus tree as needed.
#[derive(Debug)]
pub enum Action {
    /// Add a new connection with the given name and type.
    AddConnection(String, DeviceType),
    /// Update a connection (replacing the old one).
    UpdateConnection(Connection),
    /// Remove the connection with the given Uuid.
    RemoveConnection(Uuid),
    /// Apply the current configuration.
    Apply,
}
