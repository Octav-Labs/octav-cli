use serde_json::Value;

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::types::strip_portfolio_fields;
use crate::validation;

pub fn airdrop(client: &OctavClient, address: &str) -> Result<Value, OctavError> {
    validation::validate_address(address)?;
    client.get_airdrop(address)
}

pub fn polymarket(client: &OctavClient, address: &str) -> Result<Value, OctavError> {
    validation::validate_address(address)?;
    client.get_polymarket(address)
}

pub fn agent_wallet(
    client: &OctavClient,
    addresses: &[String],
    raw: bool,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    let mut data = client.get_agent_wallet(addresses)?;
    if !raw {
        strip_portfolio_fields(&mut data);
    }
    Ok(data)
}

pub fn agent_portfolio(
    client: &OctavClient,
    addresses: &[String],
    raw: bool,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    let mut data = client.get_agent_portfolio(addresses)?;
    if !raw {
        strip_portfolio_fields(&mut data);
    }
    Ok(data)
}
