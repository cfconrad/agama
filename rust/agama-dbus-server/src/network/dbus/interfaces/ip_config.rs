//! Network D-Bus interfaces for IP configuration.
//!
//! This module contains the D-Bus interfaces to deal with IPv4 and IPv6 configuration.
//! The `dbus_interface` macro should be applied to structs, that's the reason there are
//! two different structs for IPv4 and IPv6 settings. The common code have been moved
//! to the `Ip<T>` struct.
use crate::network::{
    action::Action,
    model::{Connection as NetworkConnection, IpConfig, Ipv4Method, Ipv6Method},
};
use cidr::IpInet;
use std::{net::IpAddr, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use zbus::dbus_interface;

/// D-Bus interface for IPv4 and IPv6 settings
pub struct Ip {
    actions: Arc<Mutex<UnboundedSender<Action>>>,
    connection: Arc<Mutex<NetworkConnection>>,
}

impl Ip {
    /// Creates an IP interface object.
    ///
    /// * `actions`: sending-half of a channel to send actions.
    /// * `connection`: connection to expose over D-Bus.
    pub fn new(
        actions: UnboundedSender<Action>,
        connection: Arc<Mutex<NetworkConnection>>,
    ) -> Self {
        Self {
            actions: Arc::new(Mutex::new(actions)),
            connection,
        }
    }

    /// Returns the underlying connection.
    async fn get_connection(&self) -> MutexGuard<NetworkConnection> {
        self.connection.lock().await
    }

    /// Updates the connection data in the NetworkSystem.
    ///
    /// * `connection`: Updated connection.
    async fn update_connection<'a>(
        &self,
        connection: MutexGuard<'a, NetworkConnection>,
    ) -> zbus::fdo::Result<()> {
        let actions = self.actions.lock().await;
        actions
            .send(Action::UpdateConnection(connection.clone()))
            .unwrap();
        Ok(())
    }
}

impl Ip {
    /// Returns the IpConfig struct.
    async fn get_ip_config(&self) -> MappedMutexGuard<IpConfig> {
        MutexGuard::map(self.get_connection().await, |c| &mut c.ip_config)
    }

    /// Updates the IpConfig struct.
    ///
    /// * `func`: function to update the configuration.
    async fn update_config<F>(&self, func: F) -> zbus::fdo::Result<()>
    where
        F: Fn(&mut IpConfig),
    {
        let mut connection = self.get_connection().await;
        func(&mut connection.ip_config);
        self.update_connection(connection).await?;
        Ok(())
    }
}

#[dbus_interface(name = "org.opensuse.Agama1.Network.Connection.IP")]
impl Ip {
    /// List of IP addresses.
    ///
    /// When the method is 'auto', these addresses are used as additional addresses.
    #[dbus_interface(property)]
    pub async fn addresses(&self) -> Vec<String> {
        let ip_config = self.get_ip_config().await;
        ip_config.addresses.iter().map(|a| a.to_string()).collect()
    }

    #[dbus_interface(property)]
    pub async fn set_addresses(&mut self, addresses: Vec<String>) -> zbus::fdo::Result<()> {
        let addresses = helpers::parse_addresses::<IpInet>(addresses);
        self.update_config(|ip| ip.addresses = addresses.clone())
            .await
    }

    /// IPv4 configuration method.
    ///
    /// Possible values: "disabled", "auto", "manual" or "link-local".
    ///
    /// See [crate::network::model::Ipv4Method].
    #[dbus_interface(property)]
    pub async fn method4(&self) -> String {
        let ip_config = self.get_ip_config().await;
        ip_config.method4.to_string()
    }

    #[dbus_interface(property)]
    pub async fn set_method4(&mut self, method: &str) -> zbus::fdo::Result<()> {
        let method: Ipv4Method = method.parse()?;
        self.update_config(|ip| ip.method4 = method).await
    }

    /// IPv6 configuration method.
    ///
    /// Possible values: "disabled", "auto", "manual", "link-local", "ignore" or "dhcp".
    ///
    /// See [crate::network::model::Ipv6Method].
    #[dbus_interface(property)]
    pub async fn method6(&self) -> String {
        let ip_config = self.get_ip_config().await;
        ip_config.method6.to_string()
    }

    #[dbus_interface(property)]
    pub async fn set_method6(&mut self, method: &str) -> zbus::fdo::Result<()> {
        let method: Ipv6Method = method.parse()?;
        self.update_config(|ip| ip.method6 = method).await
    }

    /// Name server addresses.
    #[dbus_interface(property)]
    pub async fn nameservers(&self) -> Vec<String> {
        let ip_config = self.get_ip_config().await;
        ip_config
            .nameservers
            .iter()
            .map(IpAddr::to_string)
            .collect()
    }

    #[dbus_interface(property)]
    pub async fn set_nameservers(&mut self, addresses: Vec<String>) -> zbus::fdo::Result<()> {
        let addresses = helpers::parse_addresses::<IpAddr>(addresses);
        self.update_config(|ip| ip.nameservers = addresses.clone())
            .await
    }

    /// Network gateway for IPv4.
    ///
    /// An empty string removes the current value.
    #[dbus_interface(property)]
    pub async fn gateway4(&self) -> String {
        let ip_config = self.get_ip_config().await;
        match ip_config.gateway4 {
            Some(ref address) => address.to_string(),
            None => "".to_string(),
        }
    }

    #[dbus_interface(property)]
    pub async fn set_gateway4(&mut self, gateway: String) -> zbus::fdo::Result<()> {
        let gateway = helpers::parse_gateway(gateway)?;
        self.update_config(|ip| ip.gateway4 = gateway).await
    }

    /// Network gateway for IPv6.
    ///
    /// An empty string removes the current value.
    #[dbus_interface(property)]
    pub async fn gateway6(&self) -> String {
        let ip_config = self.get_ip_config().await;
        match ip_config.gateway6 {
            Some(ref address) => address.to_string(),
            None => "".to_string(),
        }
    }

    #[dbus_interface(property)]
    pub async fn set_gateway6(&mut self, gateway: String) -> zbus::fdo::Result<()> {
        let gateway = helpers::parse_gateway(gateway)?;
        self.update_config(|ip| ip.gateway6 = gateway).await
    }
}

mod helpers {
    use crate::network::error::NetworkStateError;
    use log;
    use std::{
        fmt::{Debug, Display},
        str::FromStr,
    };

    /// Parses a set of addresses in textual form into T.
    ///
    /// * `addresses`: addresses to parse.
    pub fn parse_addresses<T>(addresses: Vec<String>) -> Vec<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        addresses
            .into_iter()
            .filter_map(|ip| match ip.parse::<T>() {
                Ok(address) => Some(address),
                Err(error) => {
                    log::error!("Ignoring the invalid IP address: {} ({})", ip, error);
                    None
                }
            })
            .collect()
    }

    /// Sets the gateway for an IP configuration.
    ///
    /// * `ip`: IpConfig object.
    /// * `gateway`: IP in textual form.
    pub fn parse_gateway<T>(gateway: String) -> Result<Option<T>, NetworkStateError>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug + Display,
    {
        if gateway.is_empty() {
            Ok(None)
        } else {
            let parsed = gateway
                .parse()
                .map_err(|_| NetworkStateError::InvalidIpAddr(gateway))?;
            Ok(Some(parsed))
        }
    }
}
