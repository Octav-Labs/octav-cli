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

pub fn chains(client: &OctavClient) -> Result<Value, OctavError> {
    client.get_chains()
}

pub fn chain_protocols(
    client: &OctavClient,
    chain: &str,
    page: u32,
    limit: u32,
) -> Result<Value, OctavError> {
    validation::validate_path_segment("chain", chain)?;
    client.get_chain_protocols(chain, page, limit)
}

pub fn contract_protocol(
    client: &OctavClient,
    contract: &str,
    chain: Option<&str>,
) -> Result<Value, OctavError> {
    validation::validate_address(contract)?;
    client.get_contract_protocol(contract, chain)
}
