use serde_json::Value;

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::validation;

pub fn credits(client: &OctavClient) -> Result<Value, OctavError> {
    client.get_credits()
}

pub fn status(client: &OctavClient, addresses: &[String]) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.get_status(addresses)
}
