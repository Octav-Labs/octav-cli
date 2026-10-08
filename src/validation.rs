use regex::Regex;
use std::sync::LazyLock;

use crate::error::OctavError;

static EVM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap());
static SOLANA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").unwrap());
static DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
// No '.' or '/', so a value can't escape its URL path segment
static PATH_SEGMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_:-]+$").unwrap());

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

pub fn validate_evm_address(addr: &str) -> Result<(), OctavError> {
    if EVM_REGEX.is_match(addr) {
        Ok(())
    } else {
        Err(OctavError::Validation(format!(
            "Invalid address format: '{}'. Must be an EVM (0x...) address.",
            addr
        )))
    }
}

/// Validate a value that is interpolated into the URL path (chain keys, bundle IDs,
/// address book addresses, which may also be Starknet or Tron)
pub fn validate_path_segment(kind: &str, value: &str) -> Result<(), OctavError> {
    if PATH_SEGMENT_REGEX.is_match(value) {
        Ok(())
    } else {
        Err(OctavError::Validation(format!(
            "Invalid {}: '{}'. Only letters, digits, '-', '_' and ':' are allowed.",
            kind, value
        )))
    }
}

/// Validate addresses for the address book and bundles, which accept more
/// address formats than the data endpoints and up to 100 per request
pub fn validate_address_list(addresses: &[String], max: usize) -> Result<(), OctavError> {
    if addresses.is_empty() {
        return Err(OctavError::Validation(
            "At least one address is required.".to_string(),
        ));
    }
    if addresses.len() > max {
        return Err(OctavError::Validation(format!(
            "Maximum {} addresses allowed.",
            max
        )));
    }
    for (i, addr) in addresses.iter().enumerate() {
        validate_path_segment("address", addr)?;
        if addresses[..i].contains(addr) {
            return Err(OctavError::Validation(format!(
                "Duplicate address: '{}'.",
                addr
            )));
        }
    }
    Ok(())
}

/// Refuse a command that deletes or overwrites account data unless `--yes` was passed.
/// A flag rather than an interactive prompt, so agents get an error instead of hanging.
pub fn require_confirmation(yes: bool, effect: &str) -> Result<(), OctavError> {
    if yes {
        Ok(())
    } else {
        Err(OctavError::ConfirmationRequired(format!(
            "{}. Re-run with --yes to confirm.",
            effect
        )))
    }
}

pub fn validate_virtual_addresses(addresses: &[String]) -> Result<(), OctavError> {
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
        match addr.strip_prefix("virtual:") {
            Some(id) if PATH_SEGMENT_REGEX.is_match(id) => {}
            _ => {
                return Err(OctavError::Validation(format!(
                    "Invalid virtual user address: '{}'. Must be virtual:<id> (see `octav virtual-users list`).",
                    addr
                )))
            }
        }
    }
    Ok(())
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

    #[test]
    fn test_evm_address() {
        assert!(validate_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68").is_ok());
        assert!(validate_evm_address("7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU").is_err());
    }

    #[test]
    fn test_path_segment() {
        assert!(validate_path_segment("chain", "ethereum").is_ok());
        assert!(validate_path_segment("chain", "zksync-era").is_ok());
        assert!(validate_path_segment("bundle ID", "65f1c0ab12_x").is_ok());
        assert!(validate_path_segment("bundle ID", "..").is_err());
        assert!(validate_path_segment("bundle ID", "a/b").is_err());
        assert!(validate_path_segment("bundle ID", "a?b=c").is_err());
        assert!(validate_path_segment("bundle ID", "").is_err());
    }

    #[test]
    fn test_address_list() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68".to_string();
        assert!(validate_address_list(std::slice::from_ref(&addr), 100).is_ok());
        assert!(validate_address_list(&[], 100).is_err());
        assert!(validate_address_list(&[addr.clone(), addr.clone()], 100).is_err());
        assert!(validate_address_list(&[addr.clone(), "".to_string()], 100).is_err());
        let many: Vec<String> = (0..101).map(|i| format!("0x{:040x}", i)).collect();
        assert!(validate_address_list(&many, 100).is_err());
    }

    #[test]
    fn test_require_confirmation() {
        assert!(require_confirmation(true, "This deletes things").is_ok());
        match require_confirmation(false, "This deletes things") {
            Err(OctavError::ConfirmationRequired(msg)) => {
                assert_eq!(msg, "This deletes things. Re-run with --yes to confirm.")
            }
            other => panic!("expected ConfirmationRequired, got {:?}", other),
        }
    }

    #[test]
    fn test_virtual_addresses() {
        assert!(validate_virtual_addresses(&["virtual:abc123".to_string()]).is_ok());
        assert!(validate_virtual_addresses(&["abc123".to_string()]).is_err());
        assert!(validate_virtual_addresses(&["virtual:".to_string()]).is_err());
    }
}
