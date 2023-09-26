use agama_dbus_server::network::model::{self, IpAddress, IpMethod, Ipv4Config,
    BondingMode, MiimonConfig};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Interface {
    pub name: String,
    pub control: Control,
    pub firewall: Firewall,
    pub link: Link,
    pub ipv4: Ipv4,
    #[serde(rename = "ipv4-static", skip_serializing_if = "Option::is_none")]
    pub ipv4_static: Option<Ipv4Static>,
    pub ipv6: Ipv6,
    #[serde(rename = "ipv6-static", skip_serializing_if = "Option::is_none")]
    pub ipv6_static: Option<Ipv6Static>,

//    #[serde(rename = "ipv6-dhcp", skip_serializing_if = "Option::is_none")]
//    pub ipv6_dhcp: Option<Ipv6Static>,

    #[serde(rename = "ipv4-dhcp", skip_serializing_if = "Option::is_none")]
    pub ipv4_dhcp: Option<Ipv4Dhcp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond: Option<Bond>,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Control {
    pub mode: String,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Firewall {}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<String>,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ipv4 {
    pub enabled: bool,
    #[serde(rename = "arp-verify")]
    pub arp_verify: bool,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ipv6 {
    pub enabled: bool,
    pub privacy: String,
    #[serde(rename = "accept-redir")]
    pub accept_redirects: bool,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ipv4Static {
    pub address: Address,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ipv4Dhcp {
    pub enabled: bool,
    pub flags: String,
    pub update: String,
    #[serde(rename = "defer-timeout")]
    pub defer_timeout: i32,
    #[serde(rename = "recover-lease")]
    pub recover_lease: bool,
    #[serde(rename = "release-lease")]
    pub release_lease: bool,
}


#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ipv6Static {
    pub address: Address,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Address {
    pub local: String,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Bond {
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miimon: Option<Miimon>,
    #[serde(deserialize_with = "unwrap_slaves")]
    pub slaves: Vec<Slave>,
}

impl Bond {
    pub fn primary(self: &Bond) -> Option<&String> {
        for s in self.slaves.iter() {
            if s.primary.is_some() && s.primary.unwrap(){
                return Some(&s.device);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Slave {
    pub device: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Miimon {
    pub frequency: u32,
    #[serde(rename = "carrier-detect")]
    pub carrier_detect: String,
}

fn unwrap_slaves<'de, D>(deserializer: D) -> Result<Vec<Slave>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
    struct Slaves {
        // default allows empty list
        #[serde(default)]
        slave: Vec<Slave>,
    }
    Ok(Slaves::deserialize(deserializer)?.slave)
}

impl Into<model::Connection> for Interface {
    fn into(self) -> model::Connection {

        let base = model::BaseConnection {
                id: self.name.clone(),
                interface: self.name.clone(),
                ipv4: (&self).into(),
                master: (&self).link.master.clone(),
                ..Default::default()
        };

        if let Some(b) = &self.bond {

            let mut bonding = model::BondingConfig {
                primary: match b.primary() {
                    Some(x) => Some(x.clone()),
                    _ => None
                },
                ..Default::default()
            };

            bonding.mode =  BondingMode::try_from(b.mode.as_str()).unwrap();

            if let Some(m) = &b.miimon {
                bonding.miimon = Some(MiimonConfig {
                    frequency: m.frequency,
                    ..Default::default()
                });
            }

            return model::Connection::Bonding(model::BondingConnection {
                base,
                bonding,
                ..Default::default()
            })

        } else {
            return model::Connection::Ethernet(model::EthernetConnection {
                base,
                ..Default::default()
            });
        }
    }
}

impl From<&Interface> for Ipv4Config {
    fn from(i: &Interface) -> Ipv4Config {
        let method = if i.ipv4.enabled && i.ipv4_static.is_some() {
            "manual"
        } else if !i.ipv4.enabled {
            "disabled"
        } else {
            "auto"
        };
        let method = IpMethod::from_str(method).unwrap();

        let addresses = if i.ipv4_static.is_some() {
                vec![
                    IpAddress::from_str(i.ipv4_static.as_ref().unwrap().address.local.as_str()).unwrap(),
                ]
        } else {
            vec![]
        };

        Ipv4Config { method, addresses, ..Default::default() }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_interface_to_connection() {
        let static_interface = Interface {
            ipv4: Ipv4 {
                enabled: true,
                ..Default::default()
            },
            ipv4_static: Some(Ipv4Static {
                address: Address {
                    local: "127.0.0.1/8".to_string(),
                },
            }),
            ..Default::default()
        };

        let static_connection: model::Connection = static_interface.into();
        assert_eq!(static_connection.base().ipv4.method, IpMethod::Manual);
        assert_eq!(
            static_connection.base().ipv4.addresses[0].to_string(),
            "127.0.0.1/8"
        );
    }

    #[test]
    fn test_dhcp_interface_to_connection() {
        let static_interface = Interface {
            ipv4: Ipv4 {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let static_connection: model::Connection = static_interface.into();
        assert_eq!(static_connection.base().ipv4.method, IpMethod::Auto);
        assert_eq!(static_connection.base().ipv4.addresses.len(), 0);
    }
}
