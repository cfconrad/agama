//! Representation of the network configuration
//!
//! * This module contains the types that represent the network concepts. They are supposed to be
//! agnostic from the real network service (e.g., NetworkManager).
use crate::network::error::NetworkStateError;
use agama_lib::network::types::{BondMode, DeviceType, SSID};
use cidr::IpInet;
use std::{
    collections::HashMap,
    default::Default,
    fmt,
    net::IpAddr,
    str::{self, FromStr},
};
use thiserror::Error;
use uuid::Uuid;
use zbus::zvariant::Value;
mod builder;
pub use builder::ConnectionBuilder;

#[derive(Default, Clone, Debug)]
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
    /// * `id`: connection UUID
    pub fn get_connection_by_uuid(&self, uuid: Uuid) -> Option<&Connection> {
        self.connections.iter().find(|c| c.uuid() == uuid)
    }

    /// Get connection by interface
    ///
    /// * `name`: connection interface name
    pub fn get_connection_by_interface(&self, name: &str) -> Option<&Connection> {
        let interface = Some(name);
        self.connections.iter().find(|c| c.interface() == interface)
    }

    /// Get connection by ID
    ///
    /// * `id`: connection ID
    pub fn get_connection(&self, id: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id() == id)
    }

    /// Get connection by ID as mutable
    ///
    /// * `id`: connection ID
    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id() == id)
    }

    pub fn get_controlled_by(&mut self, uuid: Uuid) -> Vec<&Connection> {
        let uuid = Some(uuid);
        self.connections
            .iter()
            .filter(|c| c.controller() == uuid)
            .collect()
    }

    /// Adds a new connection.
    ///
    /// It uses the `id` to decide whether the connection already exists.
    pub fn add_connection(&mut self, conn: Connection) -> Result<(), NetworkStateError> {
        if self.get_connection(conn.id()).is_some() {
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

    /// Sets a controller's ports.
    ///
    /// If the connection is not a controller, returns an error.
    ///
    /// * `controller`: controller to set ports on.
    /// * `ports`: list of port names (using the connection ID or the interface name).
    pub fn set_ports(
        &mut self,
        controller: &Connection,
        ports: Vec<String>,
    ) -> Result<(), NetworkStateError> {
        if let Connection::Bond(_) = &controller {
            let mut controlled = vec![];
            for port in ports {
                let connection = self
                    .get_connection_by_interface(&port)
                    .or_else(|| self.get_connection(&port))
                    .ok_or(NetworkStateError::UnknownConnection(port))?;
                controlled.push(connection.uuid());
            }

            for conn in self.connections.iter_mut() {
                if controlled.contains(&conn.uuid()) {
                    conn.set_controller(controller.uuid());
                } else if conn.controller() == Some(controller.uuid()) {
                    conn.unset_controller();
                }
            }
            Ok(())
        } else {
            Err(NetworkStateError::NotControllerConnection(
                controller.id().to_owned(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builder::ConnectionBuilder;
    use super::*;
    use crate::network::error::NetworkStateError;
    use uuid::Uuid;

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

    #[test]
    fn test_is_loopback() {
        let base = BaseConnection {
            id: "eth0".to_string(),
            ..Default::default()
        };
        let conn = Connection::Ethernet(EthernetConnection { base });
        assert!(!conn.is_loopback());

        let base = BaseConnection {
            id: "lo".to_string(),
            ..Default::default()
        };
        let conn = Connection::Loopback(LoopbackConnection { base });
        assert!(conn.is_loopback());
    }

    #[test]
    fn test_set_bonding_ports() {
        let mut state = NetworkState::default();
        let eth0 = ConnectionBuilder::new("eth0")
            .with_interface("eth0")
            .build();
        let eth1 = ConnectionBuilder::new("eth1")
            .with_interface("eth1")
            .build();
        let bond0 = ConnectionBuilder::new("bond0")
            .with_type(DeviceType::Bond)
            .build();

        state.add_connection(eth0).unwrap();
        state.add_connection(eth1).unwrap();
        state.add_connection(bond0.clone()).unwrap();

        state.set_ports(&bond0, vec!["eth1".to_string()]).unwrap();

        let eth1_found = state.get_connection("eth1").unwrap();
        assert_eq!(eth1_found.controller(), Some(bond0.uuid()));
        let eth0_found = state.get_connection("eth0").unwrap();
        assert_eq!(eth0_found.controller(), None);
    }

    #[test]
    fn test_set_bonding_missing_port() {
        let mut state = NetworkState::default();
        let bond0 = ConnectionBuilder::new("bond0")
            .with_type(DeviceType::Bond)
            .build();
        state.add_connection(bond0.clone()).unwrap();

        let error = state
            .set_ports(&bond0, vec!["eth0".to_string()])
            .unwrap_err();
        dbg!(&error);
        assert!(matches!(error, NetworkStateError::UnknownConnection(_)));
    }

    #[test]
    fn test_set_non_controller_ports() {
        let mut state = NetworkState::default();
        let eth0 = ConnectionBuilder::new("eth0").build();
        state.add_connection(eth0.clone()).unwrap();

        let error = state
            .set_ports(&eth0, vec!["eth1".to_string()])
            .unwrap_err();
        assert!(matches!(
            error,
            NetworkStateError::NotControllerConnection(_),
        ));
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
    Dummy(DummyConnection),
    Bond(BondConnection),
}

impl Connection {
    pub fn new(id: String, device_type: DeviceType) -> Self {
        let base = BaseConnection {
            id,
            uuid: Uuid::new_v4(),
            ..Default::default()
        };
        match device_type {
            DeviceType::Wireless => Connection::Wireless(WirelessConnection {
                base,
                ..Default::default()
            }),
            DeviceType::Loopback => Connection::Loopback(LoopbackConnection { base }),
            DeviceType::Ethernet => Connection::Ethernet(EthernetConnection { base }),
            DeviceType::Dummy => Connection::Dummy(DummyConnection { base }),
            DeviceType::Bond => Connection::Bond(BondConnection {
                base,
                ..Default::default()
            }),
        }
    }

    /// TODO: implement a macro to reduce the amount of repetitive code. The same applies to
    /// the base_mut function.
    pub fn base(&self) -> &BaseConnection {
        match &self {
            Connection::Ethernet(conn) => &conn.base,
            Connection::Wireless(conn) => &conn.base,
            Connection::Loopback(conn) => &conn.base,
            Connection::Dummy(conn) => &conn.base,
            Connection::Bond(conn) => &conn.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseConnection {
        match self {
            Connection::Ethernet(conn) => &mut conn.base,
            Connection::Wireless(conn) => &mut conn.base,
            Connection::Loopback(conn) => &mut conn.base,
            Connection::Dummy(conn) => &mut conn.base,
            Connection::Bond(conn) => &mut conn.base,
        }
    }

    pub fn id(&self) -> &str {
        self.base().id.as_str()
    }

    pub fn set_id(&mut self, id: &str) {
        self.base_mut().id = id.to_string()
    }

    pub fn interface(&self) -> Option<&str> {
        self.base().interface.as_deref()
    }

    pub fn set_interface(&mut self, interface: &str) {
        self.base_mut().interface = Some(interface.to_string())
    }

    /// A port's controller name, e.g.: bond0, br0
    pub fn controller(&self) -> Option<Uuid> {
        self.base().controller
    }

    /// Sets the port's controller.
    ///
    /// `controller`: Uuid of the controller (Bridge, Bond, Team), e.g.: bond0.
    pub fn set_controller(&mut self, controller: Uuid) {
        self.base_mut().controller = Some(controller)
    }

    pub fn unset_controller(&mut self) {
        self.base_mut().controller = None;
    }

    pub fn uuid(&self) -> Uuid {
        self.base().uuid
    }

    pub fn ip_config(&self) -> &IpConfig {
        &self.base().ip_config
    }

    pub fn ip_config_mut(&mut self) -> &mut IpConfig {
        &mut self.base_mut().ip_config
    }

    pub fn match_config(&self) -> &MatchConfig {
        &self.base().match_config
    }

    pub fn match_config_mut(&mut self) -> &mut MatchConfig {
        &mut self.base_mut().match_config
    }

    pub fn remove(&mut self) {
        self.base_mut().status = Status::Removed;
    }

    pub fn is_removed(&self) -> bool {
        self.base().status == Status::Removed
    }

    pub fn is_up(&self) -> bool {
        self.base().status == Status::Up
    }

    pub fn set_up(&mut self) {
        self.base_mut().status = Status::Up
    }

    pub fn set_down(&mut self) {
        self.base_mut().status = Status::Down
    }

    /// Determines whether it is a loopback interface.
    pub fn is_loopback(&self) -> bool {
        matches!(self, Connection::Loopback(_))
    }

    pub fn is_ethernet(&self) -> bool {
        matches!(self, Connection::Loopback(_))
            || matches!(self, Connection::Ethernet(_))
            || matches!(self, Connection::Dummy(_))
            || matches!(self, Connection::Bond(_))
    }

    pub fn mac_address(&self) -> String {
        self.base().mac_address.to_string()
    }

    pub fn set_mac_address(&mut self, mac_address: MacAddress) {
        self.base_mut().mac_address = mac_address;
    }
}

#[derive(Debug, Default, Clone)]
pub struct BaseConnection {
    pub id: String,
    pub uuid: Uuid,
    pub mac_address: MacAddress,
    pub ip_config: IpConfig,
    pub status: Status,
    pub interface: Option<String>,
    pub controller: Option<Uuid>,
    pub match_config: MatchConfig,
}

impl PartialEq for BaseConnection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.uuid == other.uuid && self.ip_config == other.ip_config
    }
}

#[derive(Debug, Error)]
#[error("Invalid MAC address: {0}")]
pub struct InvalidMacAddress(String);

#[derive(Debug, Default, Clone)]
pub enum MacAddress {
    MacAddress(macaddr::MacAddr6),
    Preserve,
    Permanent,
    Random,
    Stable,
    #[default]
    Unset,
}

impl FromStr for MacAddress {
    type Err = InvalidMacAddress;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "preserve" => Ok(Self::Preserve),
            "permanent" => Ok(Self::Permanent),
            "random" => Ok(Self::Random),
            "stable" => Ok(Self::Stable),
            "" => Ok(Self::Unset),
            _ => Ok(Self::MacAddress(match macaddr::MacAddr6::from_str(s) {
                Ok(mac) => mac,
                Err(e) => return Err(InvalidMacAddress(e.to_string())),
            })),
        }
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match &self {
            Self::MacAddress(mac) => mac.to_string(),
            Self::Preserve => "preserve".to_string(),
            Self::Permanent => "permanent".to_string(),
            Self::Random => "random".to_string(),
            Self::Stable => "stable".to_string(),
            Self::Unset => "".to_string(),
        };
        write!(f, "{}", output)
    }
}

impl From<InvalidMacAddress> for zbus::fdo::Error {
    fn from(value: InvalidMacAddress) -> Self {
        zbus::fdo::Error::Failed(value.to_string())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Status {
    #[default]
    Up,
    Down,
    Removed,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct IpConfig {
    pub method4: Ipv4Method,
    pub method6: Ipv6Method,
    pub addresses: Vec<IpInet>,
    pub nameservers: Vec<IpAddr>,
    pub gateway4: Option<IpAddr>,
    pub gateway6: Option<IpAddr>,
    pub routes4: Option<Vec<IpRoute>>,
    pub routes6: Option<Vec<IpRoute>>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct MatchConfig {
    pub driver: Vec<String>,
    pub interface: Vec<String>,
    pub path: Vec<String>,
    pub kernel: Vec<String>,
}

#[derive(Debug, Error)]
#[error("Unknown IP configuration method name: {0}")]
pub struct UnknownIpMethod(String);

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Ipv4Method {
    #[default]
    Disabled = 0,
    Auto = 1,
    Manual = 2,
    LinkLocal = 3,
}

impl fmt::Display for Ipv4Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self {
            Ipv4Method::Disabled => "disabled",
            Ipv4Method::Auto => "auto",
            Ipv4Method::Manual => "manual",
            Ipv4Method::LinkLocal => "link-local",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Ipv4Method {
    type Err = UnknownIpMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(Ipv4Method::Disabled),
            "auto" => Ok(Ipv4Method::Auto),
            "manual" => Ok(Ipv4Method::Manual),
            "link-local" => Ok(Ipv4Method::LinkLocal),
            _ => Err(UnknownIpMethod(s.to_string())),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Ipv6Method {
    #[default]
    Disabled = 0,
    Auto = 1,
    Manual = 2,
    LinkLocal = 3,
    Ignore = 4,
    Dhcp = 5,
}

impl fmt::Display for Ipv6Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self {
            Ipv6Method::Disabled => "disabled",
            Ipv6Method::Auto => "auto",
            Ipv6Method::Manual => "manual",
            Ipv6Method::LinkLocal => "link-local",
            Ipv6Method::Ignore => "ignore",
            Ipv6Method::Dhcp => "dhcp",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Ipv6Method {
    type Err = UnknownIpMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(Ipv6Method::Disabled),
            "auto" => Ok(Ipv6Method::Auto),
            "manual" => Ok(Ipv6Method::Manual),
            "link-local" => Ok(Ipv6Method::LinkLocal),
            "ignore" => Ok(Ipv6Method::Ignore),
            "dhcp" => Ok(Ipv6Method::Dhcp),
            _ => Err(UnknownIpMethod(s.to_string())),
        }
    }
}

impl From<UnknownIpMethod> for zbus::fdo::Error {
    fn from(value: UnknownIpMethod) -> zbus::fdo::Error {
        zbus::fdo::Error::Failed(value.to_string())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IpRoute {
    pub destination: IpInet,
    pub next_hop: Option<IpAddr>,
    pub metric: Option<u32>,
}

impl From<&IpRoute> for HashMap<&str, Value<'_>> {
    fn from(route: &IpRoute) -> Self {
        let mut map: HashMap<&str, Value> = HashMap::from([
            ("dest", Value::new(route.destination.address().to_string())),
            (
                "prefix",
                Value::new(route.destination.network_length() as u32),
            ),
        ]);
        if let Some(next_hop) = route.next_hop {
            map.insert("next-hop", Value::new(next_hop.to_string()));
        }
        if let Some(metric) = route.metric {
            map.insert("metric", Value::new(metric));
        }
        map
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
pub struct DummyConnection {
    pub base: BaseConnection,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct BondConnection {
    pub base: BaseConnection,
    pub bond: BondConfig,
}

impl BondConnection {
    pub fn set_mode(&mut self, mode: BondMode) {
        self.bond.mode = mode;
    }

    pub fn set_options(&mut self, options: BondOptions) {
        self.bond.options = options;
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BondOptions(pub HashMap<String, String>);

impl TryFrom<&str> for BondOptions {
    type Error = NetworkStateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut options = HashMap::new();

        for opt in value.split_whitespace() {
            let (key, value) = opt
                .trim()
                .split_once('=')
                .ok_or(NetworkStateError::InvalidBondOptions)?;
            options.insert(key.to_string(), value.to_string());
        }

        Ok(BondOptions(options))
    }
}

impl fmt::Display for BondOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opts = &self
            .0
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>();

        write!(f, "{}", opts.join(" "))
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct BondConfig {
    pub mode: BondMode,
    pub options: BondOptions,
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
