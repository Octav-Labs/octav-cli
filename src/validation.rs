use regex::Regex;
use std::sync::LazyLock;

use crate::error::OctavError;

static EVM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap());
static SOLANA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").unwrap());
static DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());

pub fn validate_address(addr: &str) -> Result<(), OctavError> {
    if EVM_REGEX.is_match(addr) || SOLANA_REGEX.is_match(addr) {
        Ok(())
    } else {
        Err(OctavError::Validation(format!(
            "Invalid address format: '{}'. Must be EVM (0x...) or Solana (base58) address.",
            addr
        )))
    }
}

pub fn validate_addresses(addresses: &[String]) -> Result<(), OctavError> {
    if addresses.is_empty() {
        return Err(OctavError::Validation(
            "At least one address is required.".to_string(),
        ));
    }
    if addresses.len() > 10 {
        return Err(OctavError::Validation(
            "Maximum 10 addresses allowed.".to_string(),
        ));
    }
    for addr in addresses {
        validate_address(addr)?;
    }
    Ok(())
}

pub fn validate_date(date: &str) -> Result<(), OctavError> {
    if DATE_REGEX.is_match(date) {
        Ok(())
    } else {
        Err(OctavError::Validation(format!(
            "Invalid date format: '{}'. Must be YYYY-MM-DD.",
            date
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_evm_address() {
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68").is_ok());
    }

    #[test]
    fn test_valid_solana_address() {
        assert!(validate_address("7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU").is_ok());
    }

    #[test]
    fn test_invalid_address() {
        assert!(validate_address("not-an-address").is_err());
        assert!(validate_address("0x123").is_err());
        assert!(validate_address("").is_err());
    }

    #[test]
    fn test_addresses_limits() {
        assert!(validate_addresses(&[]).is_err());
        let addrs: Vec<String> = (0..11)
            .map(|_| "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68".to_string())
            .collect();
        assert!(validate_addresses(&addrs).is_err());
    }

    #[test]
    fn test_valid_date() {
        assert!(validate_date("2024-01-15").is_ok());
    }

    #[test]
    fn test_invalid_date() {
        assert!(validate_date("2024/01/15").is_err());
        assert!(validate_date("01-15-2024").is_err());
        assert!(validate_date("not-a-date").is_err());
    }
}
