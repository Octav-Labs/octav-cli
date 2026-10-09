use serde_json::{json, Value};

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::validation;

pub fn list(client: &OctavClient) -> Result<Value, OctavError> {
    client.get_bundles()
}

pub fn get(client: &OctavClient, id: &str) -> Result<Value, OctavError> {
    validation::validate_path_segment("bundle ID", id)?;
    client.get_bundle(id)
}

pub fn create(client: &OctavClient, name: &str, addresses: &[String]) -> Result<Value, OctavError> {
    validation::validate_address_list(addresses, 100)?;
    client.create_bundle(name, addresses)
}

pub fn rename(client: &OctavClient, id: &str, name: &str, yes: bool) -> Result<Value, OctavError> {
    validation::validate_path_segment("bundle ID", id)?;
    validation::require_confirmation(
        yes,
        &format!(
            "This replaces the name of bundle {} on your Octav account",
            id
        ),
    )?;
    client.rename_bundle(id, name)
}

pub fn delete(client: &OctavClient, id: &str, yes: bool) -> Result<Value, OctavError> {
    validation::validate_path_segment("bundle ID", id)?;
    validation::require_confirmation(
        yes,
        &format!(
            "This permanently deletes bundle {} from your Octav account (its addresses stay in the address book)",
            id
        ),
    )?;
    client.delete_bundle(id)?;
    Ok(json!({
        "status": "ok",
        "message": "Bundle deleted",
        "id": id
    }))
}

pub fn add_address(client: &OctavClient, id: &str, address: &str) -> Result<Value, OctavError> {
    validation::validate_path_segment("bundle ID", id)?;
    validation::validate_path_segment("address", address)?;
    client.add_bundle_address(id, address)
}

pub fn remove_address(
    client: &OctavClient,
    id: &str,
    address: &str,
    yes: bool,
) -> Result<Value, OctavError> {
    validation::validate_path_segment("bundle ID", id)?;
    validation::validate_path_segment("address", address)?;
    validation::require_confirmation(
        yes,
        &format!(
            "This removes {} from bundle {} on your Octav account (it stays in the address book)",
            address, id
        ),
    )?;
    client.remove_bundle_address(id, address)
}
