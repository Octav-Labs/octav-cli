use crate::error::OctavError;
use crate::validation::validate_addresses;

pub fn run(api_key: &str, addresses: &[String]) -> Result<(), OctavError> {
    validate_addresses(addresses)?;

    crate::tui::run(api_key, addresses).map_err(|e| OctavError::Api {
        status: 0,
        message: format!("Dashboard error: {}", e),
    })
}
