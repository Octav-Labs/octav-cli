use serde_json::Value;

use crate::cli::PortfolioOptions;
use crate::client::OctavClient;
use crate::error::OctavError;
use crate::types::strip_portfolio_fields;
use crate::validation;

pub fn list(client: &OctavClient) -> Result<Value, OctavError> {
    client.get_virtual_users()
}

pub fn portfolio(
    client: &OctavClient,
    addresses: &[String],
    aggregated: bool,
    options: &PortfolioOptions,
    raw: bool,
) -> Result<Value, OctavError> {
    validation::validate_virtual_addresses(addresses)?;
    let mut data = client.get_virtual_users_portfolio(addresses, aggregated, options)?;
    if !raw {
        strip_portfolio_fields(&mut data);
    }
    Ok(data)
}
