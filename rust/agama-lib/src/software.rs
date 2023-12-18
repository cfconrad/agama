//! Implements support for handling the software settings

mod client;
pub mod proxies;
mod settings;
mod store;

pub use client::SoftwareClient;
pub use settings::SoftwareSettings;
pub use store::SoftwareStore;
