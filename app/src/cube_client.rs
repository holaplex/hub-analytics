use cube_client::apis::{
    configuration::Configuration as CubeConfig, default_api as cube_api, Error as CubeApiError,
};
pub use cube_client::models::{
    v1_load_request::V1LoadRequest, v1_query::Query, v1_time::TimeGranularity,
};
use hub_core::{anyhow::Result, clap, thiserror};

/// Arguments for establishing a database connection
#[derive(Clone, Debug, clap::Args)]
pub struct CubeArgs {
    #[arg(long, env, default_value = "http://127.0.0.1:4000")]
    cube_base_url: String,
    #[arg(long, env)]
    cube_auth_token: String,
}
impl CubeArgs {
    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    #[must_use]
    pub fn build_client(&self) -> Client {
        Client::new(self.clone())
    }
}
#[derive(Clone, Debug)]
pub struct Client {
    cube_base_url: String,
    auth_token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CubeClientError {
    #[error("Cube API error: {0}")]
    CubeApiError(#[from] CubeApiError<cube_api::LoadV1Error>),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl Client {
    #[must_use]
    pub fn new(args: CubeArgs) -> Self {
        let CubeArgs {
            cube_base_url,
            cube_auth_token,
        } = args;

        Self {
            cube_base_url,
            auth_token: cube_auth_token,
        }
    }

    #[must_use]
    pub fn get_client_config(&self) -> CubeConfig {
        CubeConfig {
            bearer_access_token: Some(self.auth_token.clone()),
            base_path: self.cube_base_url.clone(),
            ..Default::default()
        }
    }
    /// Res
    ///
    /// # Errors
    /// This function fails if query parameters are invalid or Cube is not responding
    pub async fn query(&self, query: Query) -> Result<String, CubeClientError> {
        let request = V1LoadRequest {
            query: Some(query.build()),
            query_type: Some("multi".to_string()),
        };

        let response = cube_api::load_v1(&self.get_client_config(), Some(request)).await?;
        Ok(serde_json::to_string(&response)?)
    }
}
