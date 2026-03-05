use serde_json::Value;

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::validation;

pub fn get(client: &OctavClient, addresses: &[String], date: &str) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    validation::validate_date(date)?;
    client.get_historical(addresses, date)
}

pub fn subscribe_snapshot(
    client: &OctavClient,
    addresses: &[String],
    description: Option<&str>,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.subscribe_snapshot(addresses, description)
}
