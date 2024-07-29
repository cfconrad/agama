use super::model::LocaleConfig;
use crate::{base_http_client::BaseHTTPClient, error::ServiceError};

pub struct LocalizationHTTPClient {
    client: BaseHTTPClient,
}

impl LocalizationHTTPClient {
    pub async fn new() -> Result<Self, ServiceError> {
        Ok(Self {
            client: BaseHTTPClient::new()?,
        })
    }

    pub async fn get_config(&self) -> Result<LocaleConfig, ServiceError> {
        self.client.get("/l10n/config").await
    }

    pub async fn set_config(&self, config: &LocaleConfig) -> Result<(), ServiceError> {
        self.client.patch("/l10n/config", config).await
    }
}
