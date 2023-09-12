use cube_client::apis::{
    configuration::Configuration as CubeConfig, default_api as cube_api, Error as CubeApiError,
};
pub use cube_client::models::{
    v1_load_request::V1LoadRequest, v1_query::Query, v1_time::TimeGranularity,
};
use hub_core::{
    anyhow::{Context, Result},
    clap, thiserror,
    url::Url,
};
use serde::de::DeserializeOwned;

/// Arguments for establishing a database connection
#[derive(Clone, Debug, clap::Args)]
pub struct CubeArgs {
    #[arg(long, env, default_value = "http://127.0.0.1:4000")]
    cube_base_url: String,
    #[arg(long, env)]
    cube_auth_token: String,
}

#[derive(Clone, Debug)]
pub struct Client(CubeConfig);

#[derive(Debug, thiserror::Error)]
pub enum CubeClientError {
    #[error("Cube API error: {0}")]
    CubeApiError(#[from] CubeApiError<cube_api::LoadV1Error>),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl Client {
    /// Res
    /// Constructs a new `Client` instance from the provided arguments.
    /// # Errors
    /// This function fails if unable to parse Url from the provided arguments.
    pub fn from_args(args: &CubeArgs) -> Result<Self> {
        // It would be a good practice to validate the URL and maybe even normalize it.
        let base_url = Url::parse(&args.cube_base_url).context("Invalid Cube base URL provided")?;

        Ok(Client(CubeConfig {
            bearer_access_token: Some(args.cube_auth_token.clone()),
            base_path: base_url.to_string(),
            ..Default::default()
        }))
    }
    /// Res
    ///
    /// # Errors
    /// This function fails if query parameters are invalid or Cube is not responding
    pub async fn query<T: DeserializeOwned>(
        &self,
        query: Query,
    ) -> Result<Vec<T>, CubeClientError> {
        let request = V1LoadRequest {
            query: Some(query.build()),
            query_type: Some("multi".to_string()),
        };

        let response = cube_api::load_v1(&self.0, Some(request)).await?;

        response.results[0]
            .data
            .iter()
            .map(|value| serde_json::from_value(value.clone()))
            .collect::<Result<Vec<T>, serde_json::Error>>()
            .map_err(Into::into)
    }
}
