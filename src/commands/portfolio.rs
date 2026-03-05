use serde_json::Value;

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::types::strip_portfolio_fields;
use crate::validation;

pub fn get(client: &OctavClient, addresses: &[String], raw: bool) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    let mut data = client.get_portfolio(addresses)?;
    if !raw {
        strip_portfolio_fields(&mut data);
    }
    Ok(data)
}

pub fn wallet(client: &OctavClient, addresses: &[String], raw: bool) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    let mut data = client.get_wallet(addresses)?;
    if !raw {
        strip_portfolio_fields(&mut data);
    }
    Ok(data)
}

pub fn nav(
    client: &OctavClient,
    addresses: &[String],
    currency: &str,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.get_nav(addresses, currency)
}

pub fn token_overview(
    client: &OctavClient,
    addresses: &[String],
    date: &str,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    validation::validate_date(date)?;
    client.get_token_overview(addresses, date)
}
