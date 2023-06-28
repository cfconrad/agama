//! Representation of the network configuration
//!
//! * This module contains the types that represent the network concepts. They are supposed to be
//! agnostic from the real network service (e.g., NetworkManager).
use uuid::Uuid;

use crate::network::error::NetworkStateError;
use agama_lib::network::types::{DeviceType, SSID};
use std::{
    fmt,
    net::{AddrParseError, Ipv4Addr},
    str::{self, FromStr},
};
use thiserror::Error;

#[derive(Default)]
pub struct NetworkState {
    pub devices: Vec<Device>,
    pub connections: Vec<Connection>,
}

impl NetworkState {
    /// Returns a NetworkState struct with the given devices and connections.
    ///
    /// * `devices`: devices to include in the state.
    /// * `connections`: connections to include in the state.
    pub fn new(devices: Vec<Device>, connections: Vec<Connection>) -> Self {
        Self {
            devices,
            connections,
        }
    }

    /// Get device by name
    ///
    /// * `name`: device name
    pub fn get_device(&self, name: &str) -> Option<&Device> {
        self.devices.iter().find(|d| d.name == name)
    }

    /// Get connection by UUID
    ///
    /// * `uuid`: connection UUID
    pub fn get_connection(&self, id: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id() == id)
    }

    /// Get connection by UUID as mutable
    ///
    /// * `uuid`: connection UUID
    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id() == id)
    }

    /// Adds a new connection.
    ///
    /// It uses the `id` to decide whether the connection already exists.
    pub fn add_connection(&mut self, conn: Connection) -> Result<(), NetworkStateError> {
        if let Some(_) = self.get_connection(conn.id()) {
            return Err(NetworkStateError::ConnectionExists(conn.uuid()));
        }

        self.connections.push(conn);
        Ok(())
    }

    /// Updates a connection with a new one.
    ///
    /// It uses the `id` to decide which connection to update.
    ///
    /// Additionally, it registers the connection to be removed when the changes are applied.
    pub fn update_connection(&mut self, conn: Connection) -> Result<(), NetworkStateError> {
        let Some(old_conn) = self.get_connection_mut(conn.id()) else {
            return Err(NetworkStateError::UnknownConnection(conn.id().to_string()));
        };

        *old_conn = conn;
        Ok(())
    }

    /// Removes a connection from the state.
    ///
    /// Additionally, it registers the connection to be removed when the changes are applied.
    pub fn remove_connection(&mut self, id: &str) -> Result<(), NetworkStateError> {
        let Some(conn) = self.get_connection_mut(id) else {
            return Err(NetworkStateError::UnknownConnection(id.to_string()));
        };

        conn.remove();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{BaseConnection, Connection, EthernetConnection, NetworkState};
    use crate::network::error::NetworkStateError;

    #[test]
    fn test_add_connection() {
        let mut state = NetworkState::default();
        let uuid = Uuid::new_v4();
        let base = BaseConnection {
            id: "eth0".to_string(),
            uuid,
            ..Default::default()
        };
        let conn0 = Connection::Ethernet(EthernetConnection { base });
        state.add_connection(conn0).unwrap();
        let found = state.get_connection("eth0").unwrap();
        assert_eq!(found.uuid(), uuid);
    }

    #[test]
    fn test_add_duplicated_connection() {
        let mut state = NetworkState::default();
        let uuid = Uuid::new_v4();
        let base = BaseConnection {
            uuid,
            ..Default::default()
        };
        let conn0 = Connection::Ethernet(EthernetConnection { base });
        state.add_connection(conn0.clone()).unwrap();
        let error = state.add_connection(conn0).unwrap_err();
        assert!(matches!(error, NetworkStateError::ConnectionExists(_)));
    }

    #[test]
    fn test_update_connection() {
        let mut state = NetworkState::default();
        let base0 = BaseConnection {
            id: "eth0".to_string(),
            uuid: Uuid::new_v4(),
            ..Default::default()
        };
        let conn0 = Connection::Ethernet(EthernetConnection { base: base0 });
        state.add_connection(conn0).unwrap();

        let uuid = Uuid::new_v4();
        let base1 = BaseConnection {
            id: "eth0".to_string(),
            uuid,
            ..Default::default()
        };
        let conn2 = Connection::Ethernet(EthernetConnection { base: base1 });
        state.update_connection(conn2).unwrap();
        let found = state.get_connection("eth0").unwrap();
        assert_eq!(found.uuid(), uuid);
    }

    #[test]
    fn test_update_unknown_connection() {
        let mut state = NetworkState::default();
        let uuid = Uuid::new_v4();
        let base = BaseConnection {
            uuid,
            ..Default::default()
        };
        let conn0 = Connection::Ethernet(EthernetConnection { base });
        let error = state.update_connection(conn0).unwrap_err();
        assert!(matches!(error, NetworkStateError::UnknownConnection(_)));
    }

    #[test]
    fn test_remove_connection() {
        let mut state = NetworkState::default();
        let id = "eth0".to_string();
        let uuid = Uuid::new_v4();
        let base0 = BaseConnection {
            id,
            uuid,
            ..Default::default()
        };
        let conn0 = Connection::Ethernet(EthernetConnection { base: base0 });
        state.add_connection(conn0).unwrap();
        state.remove_connection("eth0").unwrap();
        let found = state.get_connection("eth0").unwrap();
        assert!(found.is_removed());
    }

    #[test]
    fn test_remove_unknown_connection() {
        let mut state = NetworkState::default();
        let error = state.remove_connection("eth0").unwrap_err();
        assert!(matches!(error, NetworkStateError::UnknownConnection(_)));
    }
}

/// Network device
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub type_: DeviceType,
}

/// Represents an available network connection
#[derive(Debug, PartialEq, Clone)]
pub enum Connection {
    Ethernet(EthernetConnection),
    Wireless(WirelessConnection),
    Loopback(LoopbackConnection),
}

impl Connection {
    pub fn new(id: String, device_type: DeviceType) -> Self {
        let base = BaseConnection {
            id: id.to_string(),
            ..Default::default()
        };
        match device_type {
            DeviceType::Wireless => Connection::Wireless(WirelessConnection {
                base,
                ..Default::default()
            }),
            DeviceType::Loopback => Connection::Loopback(LoopbackConnection { base }),
            DeviceType::Ethernet => Connection::Ethernet(EthernetConnection { base }),
        }
    }

    /// TODO: implement a macro to reduce the amount of repetitive code. The same applies to
    /// the base_mut function.
    pub fn base(&self) -> &BaseConnection {
        match &self {
            Connection::Ethernet(conn) => &conn.base,
            Connection::Wireless(conn) => &conn.base,
            Connection::Loopback(conn) => &conn.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseConnection {
        match self {
            Connection::Ethernet(conn) => &mut conn.base,
            Connection::Wireless(conn) => &mut conn.base,
            Connection::Loopback(conn) => &mut conn.base,
        }
    }

    pub fn id(&self) -> &str {
        self.base().id.as_str()
    }

    pub fn uuid(&self) -> Uuid {
        self.base().uuid
    }

    pub fn ipv4(&self) -> &Ipv4Config {
        &self.base().ipv4
    }

    pub fn ipv4_mut(&mut self) -> &mut Ipv4Config {
        &mut self.base_mut().ipv4
    }

    pub fn remove(&mut self) {
        self.base_mut().status = Status::Removed;
    }

    pub fn is_removed(&self) -> bool {
        self.base().status == Status::Removed
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseConnection {
    pub id: String,
    pub uuid: Uuid,
    pub ipv4: Ipv4Config,
    pub status: Status,
}

impl PartialEq for BaseConnection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.uuid == other.uuid && self.ipv4 == other.ipv4
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Status {
    #[default]
    Present,
    Removed,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Ipv4Config {
    pub method: IpMethod,
    pub addresses: Vec<IpAddress>,
    pub nameservers: Vec<Ipv4Addr>,
    pub gateway: Option<Ipv4Addr>,
}

#[derive(Debug, Error)]
#[error("Unknown IP configuration method name: {0}")]
pub struct UnknownIpMethod(String);

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum IpMethod {
    #[default]
    Disabled = 0,
    Auto = 1,
    Manual = 2,
    LinkLocal = 3,
}
impl fmt::Display for IpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self {
            IpMethod::Disabled => "disabled",
            IpMethod::Auto => "auto",
            IpMethod::Manual => "manual",
            IpMethod::LinkLocal => "link-local",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for IpMethod {
    type Err = UnknownIpMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(IpMethod::Disabled),
            "auto" => Ok(IpMethod::Auto),
            "manual" => Ok(IpMethod::Manual),
            "link-local" => Ok(IpMethod::LinkLocal),
            _ => Err(UnknownIpMethod(s.to_string())),
        }
    }
}

impl From<UnknownIpMethod> for zbus::fdo::Error {
    fn from(value: UnknownIpMethod) -> zbus::fdo::Error {
        zbus::fdo::Error::Failed(value.to_string())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct EthernetConnection {
    pub base: BaseConnection,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct WirelessConnection {
    pub base: BaseConnection,
    pub wireless: WirelessConfig,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct LoopbackConnection {
    pub base: BaseConnection,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct WirelessConfig {
    pub mode: WirelessMode,
    pub ssid: SSID,
    pub password: Option<String>,
    pub security: SecurityProtocol,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum WirelessMode {
    Unknown = 0,
    AdHoc = 1,
    #[default]
    Infra = 2,
    AP = 3,
    Mesh = 4,
}

impl TryFrom<&str> for WirelessMode {
    type Error = NetworkStateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "unknown" => Ok(WirelessMode::Unknown),
            "adhoc" => Ok(WirelessMode::AdHoc),
            "infrastructure" => Ok(WirelessMode::Infra),
            "ap" => Ok(WirelessMode::AP),
            "mesh" => Ok(WirelessMode::Mesh),
            _ => Err(NetworkStateError::InvalidWirelessMode(value.to_string())),
        }
    }
}

impl fmt::Display for WirelessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self {
            WirelessMode::Unknown => "unknown",
            WirelessMode::AdHoc => "adhoc",
            WirelessMode::Infra => "infrastructure",
            WirelessMode::AP => "ap",
            WirelessMode::Mesh => "mesh",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SecurityProtocol {
    #[default]
    WEP, // No encryption or WEP ("none")
    OWE,            // Opportunistic Wireless Encryption ("owe")
    DynamicWEP,     // Dynamic WEP ("ieee8021x")
    WPA2,           // WPA2 + WPA3 personal ("wpa-psk")
    WPA3Personal,   // WPA3 personal only ("sae")
    WPA2Enterprise, // WPA2 + WPA3 Enterprise ("wpa-eap")
    WPA3Only,       // WPA3 only ("wpa-eap-suite-b192")
}

impl fmt::Display for SecurityProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match &self {
            SecurityProtocol::WEP => "none",
            SecurityProtocol::OWE => "owe",
            SecurityProtocol::DynamicWEP => "ieee8021x",
            SecurityProtocol::WPA2 => "wpa-psk",
            SecurityProtocol::WPA3Personal => "sae",
            SecurityProtocol::WPA2Enterprise => "wpa-eap",
            SecurityProtocol::WPA3Only => "wpa-eap-suite-b192",
        };
        write!(f, "{}", value)
    }
}

impl TryFrom<&str> for SecurityProtocol {
    type Error = NetworkStateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(SecurityProtocol::WEP),
            "owe" => Ok(SecurityProtocol::OWE),
            "ieee8021x" => Ok(SecurityProtocol::DynamicWEP),
            "wpa-psk" => Ok(SecurityProtocol::WPA2),
            "sae" => Ok(SecurityProtocol::WPA3Personal),
            "wpa-eap" => Ok(SecurityProtocol::WPA2Enterprise),
            "wpa-eap-suite-b192" => Ok(SecurityProtocol::WPA3Only),
            _ => Err(NetworkStateError::InvalidSecurityProtocol(
                value.to_string(),
            )),
        }
    }
}

/// Represents an IPv4 address with a prefix.
#[derive(Debug, Clone, PartialEq)]
pub struct IpAddress(Ipv4Addr, u32);

#[derive(Error, Debug)]
pub enum ParseIpAddressError {
    #[error("Missing prefix")]
    MissingPrefix,
    #[error("Invalid prefix part '{0}'")]
    InvalidPrefix(String),
    #[error("Invalid address part: {0}")]
    InvalidAddr(AddrParseError),
}

impl IpAddress {
    /// Returns an new IpAddress object
    ///
    /// * `addr`: IPv4 address.
    /// * `prefix`: IPv4 address prefix.
    pub fn new(addr: Ipv4Addr, prefix: u32) -> Self {
        IpAddress(addr, prefix)
    }

    /// Returns the IPv4 address.
    pub fn addr(&self) -> &Ipv4Addr {
        &self.0
    }

    /// Returns the prefix.
    pub fn prefix(&self) -> u32 {
        self.1
    }
}

impl From<IpAddress> for (String, u32) {
    fn from(value: IpAddress) -> Self {
        (value.0.to_string(), value.1)
    }
}

impl FromStr for IpAddress {
    type Err = ParseIpAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((address, prefix)) = s.split_once("/") else {
            return Err(ParseIpAddressError::MissingPrefix);
        };

        let address: Ipv4Addr = address
            .parse()
            .map_err(|e| ParseIpAddressError::InvalidAddr(e))?;

        let prefix: u32 = prefix
            .parse()
            .map_err(|_| ParseIpAddressError::InvalidPrefix(prefix.to_string()))?;

        Ok(IpAddress(address, prefix))
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0.to_string(), self.1)
    }
}
