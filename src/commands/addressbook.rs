use serde_json::{json, Value};

use crate::client::OctavClient;
use crate::error::OctavError;
use crate::validation;

pub fn list(client: &OctavClient) -> Result<Value, OctavError> {
    client.get_addressbook()
}

pub fn add(
    client: &OctavClient,
    addresses: &[String],
    label: Option<&str>,
) -> Result<Value, OctavError> {
    validation::validate_address_list(addresses, 100)?;
    if label.is_some() && addresses.len() > 1 {
        return Err(OctavError::Validation(
            "--label can only be used when adding a single address.".to_string(),
        ));
    }
    client.add_addressbook_entries(addresses, label)
}

pub fn rename(
    client: &OctavClient,
    address: &str,
    label: &str,
    yes: bool,
) -> Result<Value, OctavError> {
    validation::validate_path_segment("address", address)?;
    validation::require_confirmation(
        yes,
        &format!(
            "This replaces the current label of {} in the address book on your Octav account",
            address
        ),
    )?;
    client.rename_addressbook_entry(address, label)
}

pub fn remove(client: &OctavClient, address: &str, yes: bool) -> Result<Value, OctavError> {
    validation::validate_path_segment("address", address)?;
    validation::require_confirmation(
        yes,
        &format!(
            "This permanently removes {} from the address book on your Octav account",
            address
        ),
    )?;
    client.remove_addressbook_entry(address)?;
    Ok(json!({
        "status": "ok",
        "message": "Address removed from address book",
        "address": address
    }))
}
