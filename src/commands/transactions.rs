use serde_json::Value;

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::validation;

#[allow(clippy::too_many_arguments)]
pub fn get(
    client: &OctavClient,
    addresses: &[String],
    chain: Option<&str>,
    tx_type: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    offset: u32,
    limit: u32,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    if let Some(sd) = start_date {
        validation::validate_date(sd)?;
    }
    if let Some(ed) = end_date {
        validation::validate_date(ed)?;
    }
    if limit > 250 {
        return Err(OctavError::Validation(
            "Limit must be at most 250.".to_string(),
        ));
    }
    client.get_transactions(
        addresses, chain, tx_type, start_date, end_date, offset, limit,
    )
}

pub fn sync(client: &OctavClient, addresses: &[String]) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.sync_transactions(addresses)
}
