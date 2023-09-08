use agama_lib::network::settings::NetworkConnection;
use agama_lib::network::NetworkClient;
use agama_lib::connection;
use crate::interface::Interface;

pub async fn migrate(interfaces: Vec<Interface>) {
    let network = NetworkClient::new(connection().await.unwrap()).await.unwrap();

    //debug
    println!("before: {:?}",network.connections().await.unwrap());

    for interface in interfaces {
        let nc: NetworkConnection = interface.into();
        network.add_or_update_connection(&nc).await.unwrap();
    };

    //debug
    println!("after: {:?}",network.connections().await.unwrap());

    network.apply().await.unwrap();
}
