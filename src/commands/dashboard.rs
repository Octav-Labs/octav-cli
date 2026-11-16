use crate::error::OctavError;
use crate::validation::{validate_addresses, SINGLE_ADDRESS};

pub fn run(api_key: &str, addresses: &[String]) -> Result<(), OctavError> {
    // The dashboard calls portfolio and transactions, which take one address
    validate_addresses(addresses, SINGLE_ADDRESS)?;

    crate::tui::run(api_key, addresses).map_err(|e| OctavError::Api {
        status: 0,
        message: format!("Dashboard error: {}", e),
    })
}
