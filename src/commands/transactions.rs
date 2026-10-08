use serde_json::Value;

use crate::cli::TransactionFilters;
use crate::client::OctavClient;
use crate::error::OctavError;
use crate::validation;

pub fn get(
    client: &OctavClient,
    addresses: &[String],
    filters: &TransactionFilters,
    offset: u32,
    limit: u32,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    for addr in &filters.interacting_address {
        validation::validate_address(addr)?;
    }
    if let Some(sd) = &filters.start_date {
        validation::validate_date(sd)?;
    }
    if let Some(ed) = &filters.end_date {
        validation::validate_date(ed)?;
    }
    if limit > 250 {
        return Err(OctavError::Validation(
            "Limit must be at most 250.".to_string(),
        ));
    }
    client.get_transactions(addresses, filters, offset, limit)
}

pub fn sync(client: &OctavClient, addresses: &[String]) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.sync_transactions(addresses)
}
