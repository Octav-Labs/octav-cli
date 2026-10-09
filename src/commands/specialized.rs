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

pub fn approvals(
    client: &OctavClient,
    address: &str,
    chain: &str,
    limit: u32,
    cursor: Option<&str>,
) -> Result<Value, OctavError> {
    validation::validate_evm_address(address)?;
    validation::validate_path_segment("chain", chain)?;
    client.get_approvals(address, chain, limit, cursor)
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

pub fn agent_nav(
    client: &OctavClient,
    addresses: &[String],
    currency: &str,
) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.get_agent_nav(addresses, currency)
}

pub fn agent_status(client: &OctavClient, addresses: &[String]) -> Result<Value, OctavError> {
    validation::validate_addresses(addresses)?;
    client.get_agent_status(addresses)
}

pub fn agent_chains(client: &OctavClient) -> Result<Value, OctavError> {
    client.get_agent_chains()
}
