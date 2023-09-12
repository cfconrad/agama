use crate::reader::read_dir as wicked_read_dir;
use agama_dbus_server::network::{model, Adapter, NetworkManagerAdapter, NetworkState};
use std::error::Error;
use std::path::PathBuf;

struct WickedAdapter {
    path: PathBuf,
}

impl WickedAdapter {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.into(),
        }
    }
}

impl Adapter for WickedAdapter {
    fn read(&self) -> Result<model::NetworkState, Box<dyn std::error::Error>> {
        async_std::task::block_on(async {
            let interfaces = wicked_read_dir(self.path.clone()).await?;
            let mut state = NetworkState::new(vec![], vec![]);

            for interface in interfaces {
                let conn: model::Connection = interface.into();
                state.add_connection(conn)?;
            }
            Ok(state)
        })
    }

    fn write(&self, _network: &model::NetworkState) -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!("not needed");
    }
}

pub async fn migrate(path: String) -> Result<(), Box<dyn Error>> {
    let wicked = WickedAdapter::new(&path);
    let state = wicked.read()?;
    let nm = NetworkManagerAdapter::from_system().await?;
    nm.write(&state)?;
    Ok(())
}
