use std::convert::TryFrom;

use async_graphql::SimpleObject;
use hub_core::{
    anyhow::{Error, Result},
    chrono::NaiveDateTime,
    uuid::Uuid,
};
use serde::Deserialize;
use serde_aux::prelude::*;
use serde_json::Value;
/// A `DataPoint` object containing analytics information.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
pub struct DataPoint {
    /// Count of the metric.
    #[serde(
        rename = "mints.count",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub count: u64,
    /// The ID of the organization the data belongs to.
    pub organization_id: Option<Uuid>,
    /// The ID of the collection the data belongs to.
    #[serde(rename = "mints.collection_id")]
    pub collection_id: Option<Uuid>,
    /// The ID of the project the data belongs to.
    #[serde(rename = "mints.project_id")]
    pub project_id: Option<Uuid>,
    /// The timestamp associated with the data point.
    #[serde(rename = "mints.timestamp")]
    pub timestamp: Option<NaiveDateTime>,
}

impl TryFrom<Value> for DataPoint {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let count = value["mints"]["count"].as_u64().unwrap();
        let organization_id = value["projects"]["organization_id"]
            .as_str()
            .map(|s| Uuid::parse_str(s).unwrap());
        let project_id = value["mints"]["project_id"]
            .as_str()
            .map(|s| Uuid::parse_str(s).unwrap());
        let collection_id = value["mints"]["collection_id"]
            .as_str()
            .map(|s| Uuid::parse_str(s).unwrap());

        let timestamp_str = value["collections"]["timestamp"]["minute"].as_str();
        let timestamp = match timestamp_str {
            Some(s) => NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3fZ").ok(),
            None => None,
        };

        Ok(Self {
            count,
            organization_id,
            collection_id,
            project_id,
            timestamp,
        })
    }
}
