use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::error::OctavError;

static EVM_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap());
static SOLANA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").unwrap());
static DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
// No '.' or '/', so a value can't escape its URL path segment
static PATH_SEGMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_:-]+$").unwrap());

// Address helpers that match how the Octav API validates and deduplicates addresses, so the
// CLI rejects the same inputs the API would, with a clearer message.

/// Lowercase an EVM address, keeping the `0x` prefix
pub fn normalize_evm_address(address: &str) -> String {
    let lower = address.to_lowercase();
    format!("0x{}", lower.strip_prefix("0x").unwrap_or(&lower))
}

/// An EVM address: `0x` followed by 40 hex characters. The API checks the normalized (lowercase)
/// address, so call this on `normalize_evm_address(address)`.
pub fn is_evm_address(address: &str) -> bool {
    address.len() == 42 && EVM_REGEX.is_match(address)
}

/// EVM addresses are lowercased; others (Solana, Tron, Starknet) are kept as given.
pub fn normalize_address(address: &str) -> String {
    let evm = normalize_evm_address(address);
    if is_evm_address(&evm) {
        evm
    } else {
        address.to_string()
    }
}

/// Data endpoints (nav, sync, subscribe, status, transactions) and virtual users treat two
/// addresses as the same when they differ only in case, whatever the address type.
fn has_duplicates(arr: &[String]) -> bool {
    arr.len()
        != arr
            .iter()
            .map(|a| a.to_lowercase())
            .collect::<HashSet<_>>()
            .len()
}

/// The address book and bundles compare normalized addresses exactly, since only EVM
/// addresses are case-insensitive.
fn has_duplicate_addresses(addresses: &[String]) -> bool {
    addresses.len() != addresses.iter().collect::<HashSet<_>>().len()
}

/// The first address that repeats an earlier one under `key`, for the error message
fn find_duplicate(addresses: &[String], key: impl Fn(&str) -> String) -> Option<&String> {
    let mut seen = HashSet::new();
    addresses.iter().find(|address| !seen.insert(key(address)))
}

fn duplicate_error(address: Option<&String>) -> OctavError {
    OctavError::Validation(format!(
        "Duplicate address: '{}'.",
        address.map(String::as_str).unwrap_or_default()
    ))
}

fn is_valid_evm_address(addr: &str) -> bool {
    is_evm_address(&normalize_evm_address(addr))
}

pub fn validate_address(addr: &str) -> Result<(), OctavError> {
    if is_valid_evm_address(addr) || SOLANA_REGEX.is_match(addr) {
        Ok(())
    } else {
        Err(OctavError::Validation(format!(
            "Invalid address format: '{}'. Must be EVM (0x...) or Solana (base58) address.",
            addr
        )))
    }
}

pub fn validate_evm_address(addr: &str) -> Result<(), OctavError> {
    if is_valid_evm_address(addr) {
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
    for addr in addresses {
        validate_path_segment("address", addr)?;
    }
    let normalized: Vec<String> = addresses.iter().map(|a| normalize_address(a)).collect();
    if has_duplicate_addresses(&normalized) {
        return Err(duplicate_error(find_duplicate(
            addresses,
            normalize_address,
        )));
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

/// Most wallet data endpoints take one address per call
pub const SINGLE_ADDRESS: Option<usize> = Some(1);
/// `/nav` and `/virtual-users/portfolio` accept up to 50 addresses
pub const MAX_MULTI_ADDRESSES: Option<usize> = Some(50);

fn check_address_count(addresses: &[String], max: Option<usize>) -> Result<(), OctavError> {
    if addresses.is_empty() {
        return Err(OctavError::Validation(
            "At least one address is required.".to_string(),
        ));
    }
    match max {
        Some(1) if addresses.len() > 1 => Err(OctavError::Validation(
            "This endpoint accepts one address per call. Run the command once per address."
                .to_string(),
        )),
        Some(max) if addresses.len() > max => Err(OctavError::Validation(format!(
            "Maximum {} addresses allowed.",
            max
        ))),
        _ => Ok(()),
    }
}

fn check_no_duplicates(addresses: &[String]) -> Result<(), OctavError> {
    if has_duplicates(addresses) {
        return Err(duplicate_error(find_duplicate(addresses, |a| {
            a.to_lowercase()
        })));
    }
    Ok(())
}

pub fn validate_virtual_addresses(addresses: &[String]) -> Result<(), OctavError> {
    check_address_count(addresses, MAX_MULTI_ADDRESSES)?;
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
    check_no_duplicates(addresses)
}

/// Validate wallet addresses against the endpoint's limit (`None` for no limit)
pub fn validate_addresses(addresses: &[String], max: Option<usize>) -> Result<(), OctavError> {
    check_address_count(addresses, max)?;
    for addr in addresses {
        validate_address(addr)?;
    }
    check_no_duplicates(addresses)
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
        let a = "0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68".to_string();
        let b = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string();
        assert!(validate_addresses(&[], SINGLE_ADDRESS).is_err());
        assert!(validate_addresses(std::slice::from_ref(&a), SINGLE_ADDRESS).is_ok());
        assert!(validate_addresses(&[a.clone(), b.clone()], SINGLE_ADDRESS).is_err());
        assert!(validate_addresses(&[a.clone(), b.clone()], MAX_MULTI_ADDRESSES).is_ok());
        assert!(validate_addresses(&[a.clone(), a.clone()], None).is_err());
        let many: Vec<String> = (0..51).map(|i| format!("0x{:040x}", i)).collect();
        assert!(validate_addresses(&many[..50], MAX_MULTI_ADDRESSES).is_ok());
        assert!(validate_addresses(&many, MAX_MULTI_ADDRESSES).is_err());
        assert!(validate_addresses(&many, None).is_ok());
    }

    #[test]
    fn test_server_address_rules() {
        let lo = format!("0x{}", "a".repeat(40));
        let up = format!("0x{}", "A".repeat(40));
        let sol = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".to_string();
        let sol_case = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsu".to_string();
        let v = |a: &[&String]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        // EVM addresses are normalized, then validated, as the API does
        assert_eq!(normalize_evm_address(&up), lo);
        assert!(is_evm_address(&normalize_evm_address(&up)));
        assert!(validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68").is_ok()); // mis-checksummed: normalized first
        assert!(validate_evm_address(&format!("0x{}", "a".repeat(39))).is_err());
        assert!(validate_evm_address(&sol).is_err());
        assert_eq!(normalize_address(&sol), sol);

        // hasDuplicates (data endpoints, virtual users): case-insensitive for every address
        assert!(validate_addresses(&v(&[&lo, &up]), MAX_MULTI_ADDRESSES).is_err());
        assert!(validate_addresses(&v(&[&sol, &sol_case]), MAX_MULTI_ADDRESSES).is_err());
        assert!(validate_virtual_addresses(&["virtual:abc".into(), "virtual:ABC".into()]).is_err());

        // hasDuplicateAddresses after normalizeAddress (address book, bundles)
        assert!(validate_address_list(&v(&[&lo, &up]), 100).is_err());
        assert!(validate_address_list(&v(&[&sol, &sol_case]), 100).is_ok());
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
