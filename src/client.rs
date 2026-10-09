use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};

use crate::cli::{PortfolioOptions, TransactionFilters};
use crate::error::OctavError;

type Params = Vec<(&'static str, String)>;

pub struct OctavClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl OctavClient {
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.octav.fi/v1".to_string(),
            api_key,
            client: Client::new(),
        }
    }

    fn request(
        &self,
        method: Method,
        endpoint: &str,
        params: &[(&str, String)],
        body: Option<Value>,
    ) -> Result<Value, OctavError> {
        let url = format!("{}{}", self.base_url, endpoint);

        let builder = self
            .client
            .request(method, &url)
            .query(params)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        let builder = if let Some(b) = body {
            builder.json(&b)
        } else {
            builder
        };

        let request = builder
            .build()
            .map_err(|e| OctavError::Network(format!("Invalid request: {}", e)))?;
        eprintln!("[OCTAV] {} {}", request.method(), request.url());

        let response = self
            .client
            .execute(request)
            .map_err(|e| OctavError::Network(format!("Request failed: {}", e)))?;

        let status = response.status().as_u16();

        if status >= 400 {
            let text = response.text().unwrap_or_default();
            let error_data: Value = serde_json::from_str(&text)
                .unwrap_or_else(|_| serde_json::json!({ "message": text }));
            let message = api_error_message(status, &error_data);

            return Err(match status {
                401 => OctavError::Auth(message),
                402 => {
                    OctavError::InsufficientCredits(message, error_data["creditsNeeded"].as_f64())
                }
                429 => OctavError::RateLimit(message, error_data["retryAfter"].as_u64()),
                _ => OctavError::Api { status, message },
            });
        }

        let text = response
            .text()
            .map_err(|e| OctavError::Network(format!("Failed to read response: {}", e)))?;

        // 204 No Content (e.g. DELETE /bundles/{id}); callers decide what to report
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }

        // Handle bare number responses (e.g. /credits returns just a number)
        // and bare string responses (e.g. /sync-transactions returns a quoted string)
        serde_json::from_str(&text).map_err(|_| OctavError::Api {
            status: 200,
            message: format!("Invalid JSON response: {}", text),
        })
    }

    // Portfolio endpoints
    pub fn get_portfolio(
        &self,
        addresses: &[String],
        options: &PortfolioOptions,
    ) -> Result<Value, OctavError> {
        let mut params = address_params(addresses);
        params.extend(portfolio_params(options));
        self.request(Method::GET, "/portfolio", &params, None)
    }

    pub fn get_wallet(&self, addresses: &[String]) -> Result<Value, OctavError> {
        self.request(Method::GET, "/wallet", &address_params(addresses), None)
    }

    pub fn get_nav(&self, addresses: &[String], currency: &str) -> Result<Value, OctavError> {
        let mut params = address_params(addresses);
        params.push(("currency", currency.to_string()));
        self.request(Method::GET, "/nav", &params, None)
    }

    pub fn get_token_overview(
        &self,
        addresses: &[String],
        date: &str,
    ) -> Result<Value, OctavError> {
        let mut params = address_params(addresses);
        params.push(("date", date.to_string()));
        self.request(Method::GET, "/token-overview", &params, None)
    }

    pub fn get_portfolio_at_block(
        &self,
        address: &str,
        chain: &str,
        block: u64,
    ) -> Result<Value, OctavError> {
        let params = vec![
            ("addresses", address.to_string()),
            ("chainKey", chain.to_string()),
            ("blockNumber", block.to_string()),
        ];
        self.request(Method::GET, "/portfolio/at-block", &params, None)
    }

    // Transaction endpoints
    pub fn get_transactions(
        &self,
        addresses: &[String],
        filters: &TransactionFilters,
        offset: u32,
        limit: u32,
    ) -> Result<Value, OctavError> {
        let params = transaction_params(addresses, filters, offset, limit);
        self.request(Method::GET, "/transactions", &params, None)
    }

    pub fn sync_transactions(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let body = serde_json::json!({ "addresses": addresses });
        self.request(Method::POST, "/sync-transactions", &[], Some(body))
    }

    // Historical endpoints
    pub fn get_historical(&self, addresses: &[String], date: &str) -> Result<Value, OctavError> {
        let mut params = address_params(addresses);
        params.push(("date", date.to_string()));
        self.request(Method::GET, "/historical", &params, None)
    }

    pub fn subscribe_snapshot(
        &self,
        addresses: &[String],
        description: Option<&str>,
    ) -> Result<Value, OctavError> {
        let addr_objects: Vec<Value> = addresses
            .iter()
            .map(|a| {
                let mut obj = serde_json::json!({ "address": a });
                if let Some(desc) = description {
                    obj["description"] = serde_json::json!(desc);
                }
                obj
            })
            .collect();
        let body = serde_json::json!({ "addresses": addr_objects });
        self.request(Method::POST, "/subscribe-snapshot", &[], Some(body))
    }

    // Metadata endpoints
    pub fn get_status(&self, addresses: &[String]) -> Result<Value, OctavError> {
        self.request(Method::GET, "/status", &address_params(addresses), None)
    }

    pub fn get_credits(&self) -> Result<Value, OctavError> {
        self.request(Method::GET, "/credits", &[], None)
    }

    pub fn get_chains(&self) -> Result<Value, OctavError> {
        self.request(Method::GET, "/chains", &[], None)
    }

    pub fn get_chain_protocols(
        &self,
        chain: &str,
        page: u32,
        limit: u32,
    ) -> Result<Value, OctavError> {
        let params = vec![("page", page.to_string()), ("limit", limit.to_string())];
        self.request(
            Method::GET,
            &format!("/chains/{}/protocols", chain),
            &params,
            None,
        )
    }

    pub fn get_contract_protocol(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<Value, OctavError> {
        let mut params = vec![("contract", contract.to_string())];
        if let Some(c) = chain {
            params.push(("chain", c.to_string()));
        }
        self.request(Method::GET, "/contract-protocol", &params, None)
    }

    // Specialized endpoints
    pub fn get_airdrop(&self, address: &str) -> Result<Value, OctavError> {
        let params = vec![("addresses", address.to_string())];
        self.request(Method::GET, "/airdrop", &params, None)
    }

    pub fn get_polymarket(&self, address: &str) -> Result<Value, OctavError> {
        let params = vec![("addresses", address.to_string())];
        self.request(Method::GET, "/portfolio/proxy/polymarket", &params, None)
    }

    pub fn get_approvals(
        &self,
        address: &str,
        chain: &str,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<Value, OctavError> {
        let mut params = vec![
            ("addresses", address.to_string()),
            ("limit", limit.to_string()),
        ];
        if let Some(c) = cursor {
            params.push(("cursor", c.to_string()));
        }
        self.request(Method::GET, &format!("/approvals/{}", chain), &params, None)
    }

    // Address book endpoints
    pub fn get_addressbook(&self) -> Result<Value, OctavError> {
        self.request(Method::GET, "/addressbook", &[], None)
    }

    pub fn add_addressbook_entries(
        &self,
        addresses: &[String],
        label: Option<&str>,
    ) -> Result<Value, OctavError> {
        let entries: Vec<Value> = addresses
            .iter()
            .map(|a| {
                let mut entry = json!({ "address": a });
                if let Some(l) = label {
                    entry["label"] = json!(l);
                }
                entry
            })
            .collect();
        let body = json!({ "entries": entries });
        self.request(Method::POST, "/addressbook", &[], Some(body))
    }

    pub fn rename_addressbook_entry(
        &self,
        address: &str,
        label: &str,
    ) -> Result<Value, OctavError> {
        let body = json!({ "label": label });
        self.request(
            Method::PATCH,
            &format!("/addressbook/{}", address),
            &[],
            Some(body),
        )
    }

    pub fn remove_addressbook_entry(&self, address: &str) -> Result<Value, OctavError> {
        self.request(
            Method::DELETE,
            &format!("/addressbook/{}", address),
            &[],
            None,
        )
    }

    // Bundle endpoints
    pub fn get_bundles(&self) -> Result<Value, OctavError> {
        self.request(Method::GET, "/bundles", &[], None)
    }

    pub fn get_bundle(&self, id: &str) -> Result<Value, OctavError> {
        self.request(Method::GET, &format!("/bundles/{}", id), &[], None)
    }

    pub fn create_bundle(&self, name: &str, addresses: &[String]) -> Result<Value, OctavError> {
        let body = json!({ "name": name, "addresses": addresses });
        self.request(Method::POST, "/bundles", &[], Some(body))
    }

    pub fn rename_bundle(&self, id: &str, name: &str) -> Result<Value, OctavError> {
        let body = json!({ "name": name });
        self.request(Method::PATCH, &format!("/bundles/{}", id), &[], Some(body))
    }

    pub fn delete_bundle(&self, id: &str) -> Result<Value, OctavError> {
        self.request(Method::DELETE, &format!("/bundles/{}", id), &[], None)
    }

    pub fn add_bundle_address(&self, id: &str, address: &str) -> Result<Value, OctavError> {
        let body = json!({ "address": address });
        self.request(
            Method::POST,
            &format!("/bundles/{}/addresses", id),
            &[],
            Some(body),
        )
    }

    pub fn remove_bundle_address(&self, id: &str, address: &str) -> Result<Value, OctavError> {
        self.request(
            Method::DELETE,
            &format!("/bundles/{}/addresses/{}", id, address),
            &[],
            None,
        )
    }

    // Virtual user endpoints
    pub fn get_virtual_users(&self) -> Result<Value, OctavError> {
        self.request(Method::GET, "/virtual-users", &[], None)
    }

    pub fn get_virtual_users_portfolio(
        &self,
        addresses: &[String],
        aggregated: bool,
        options: &PortfolioOptions,
    ) -> Result<Value, OctavError> {
        // Unlike other endpoints, virtual user addresses go in a single comma-separated value
        let mut params = vec![("addresses", addresses.join(","))];
        if aggregated {
            params.push(("aggregated", "true".to_string()));
        }
        params.extend(portfolio_params(options));
        self.request(Method::GET, "/virtual-users/portfolio", &params, None)
    }

    // Agent endpoints
    pub fn get_agent_wallet(&self, addresses: &[String]) -> Result<Value, OctavError> {
        self.request(
            Method::GET,
            "/agent/wallet",
            &address_params(addresses),
            None,
        )
    }

    pub fn get_agent_portfolio(&self, addresses: &[String]) -> Result<Value, OctavError> {
        self.request(
            Method::GET,
            "/agent/portfolio",
            &address_params(addresses),
            None,
        )
    }

    pub fn get_agent_nav(&self, addresses: &[String], currency: &str) -> Result<Value, OctavError> {
        let mut params = address_params(addresses);
        params.push(("currency", currency.to_string()));
        self.request(Method::GET, "/agent/nav", &params, None)
    }

    pub fn get_agent_status(&self, addresses: &[String]) -> Result<Value, OctavError> {
        self.request(
            Method::GET,
            "/agent/status",
            &address_params(addresses),
            None,
        )
    }

    pub fn get_agent_chains(&self) -> Result<Value, OctavError> {
        self.request(Method::GET, "/agent/chains", &[], None)
    }
}

fn address_params(addresses: &[String]) -> Params {
    addresses.iter().map(|a| ("addresses", a.clone())).collect()
}

fn portfolio_params(options: &PortfolioOptions) -> Params {
    let mut params = vec![("includeImages", "true".to_string())];
    if !options.no_wait_for_sync {
        params.push(("waitForSync", "true".to_string()));
    }
    if options.explorer_urls {
        params.push(("includeExplorerUrls", "true".to_string()));
    }
    params
}

fn transaction_params(
    addresses: &[String],
    filters: &TransactionFilters,
    offset: u32,
    limit: u32,
) -> Params {
    let mut params = address_params(addresses);
    let lists = [
        ("networks", &filters.chain),
        ("txTypes", &filters.tx_type),
        ("protocols", &filters.protocol),
        ("interactingAddresses", &filters.interacting_address),
    ];
    for (key, values) in lists {
        if !values.is_empty() {
            params.push((key, values.join(",")));
        }
    }
    let optional = [
        ("initialSearchText", &filters.search),
        ("tokenId", &filters.token_id),
        ("startDate", &filters.start_date),
        ("endDate", &filters.end_date),
    ];
    for (key, value) in optional {
        if let Some(v) = value {
            params.push((key, v.clone()));
        }
    }
    if let Some(sort) = &filters.sort {
        params.push(("sort", sort.to_string()));
    }
    if filters.hide_spam {
        params.push(("hideSpam", "true".to_string()));
    }
    if filters.hide_dust {
        params.push(("hideDust", "true".to_string()));
    }
    params.push(("offset", offset.to_string()));
    params.push(("limit", limit.to_string()));
    params
}

/// Build a readable message from an API error body. The API returns either
/// `{"message": ...}` or `{"error": ..., "details": {<location>: [{"message": ...}]}}`.
fn api_error_message(status: u16, data: &Value) -> String {
    let base = [&data["message"], &data["error"], &data["error"]["message"]]
        .iter()
        .find_map(|v| v.as_str().filter(|s| !s.is_empty()))
        .map(String::from)
        .unwrap_or_else(|| format!("API request failed with status {}", status));

    let details: Vec<&str> = data["details"]
        .as_object()
        .into_iter()
        .flat_map(|locations| locations.values())
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(|d| d["message"].as_str())
        .collect();

    if details.is_empty() {
        base
    } else {
        format!("{}: {}", base, details.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SortOrder;

    fn addrs() -> Vec<String> {
        vec!["0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68".to_string()]
    }

    #[test]
    fn test_transaction_params_use_api_names() {
        let filters = TransactionFilters {
            chain: vec!["arbitrum".into(), "optimism".into()],
            tx_type: vec!["SWAP".into()],
            protocol: vec!["uniswap".into()],
            search: Some("USDC".into()),
            sort: Some(SortOrder::Asc),
            hide_spam: true,
            ..Default::default()
        };
        let params = transaction_params(&addrs(), &filters, 10, 25);
        let get = |k: &str| {
            params
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.as_str())
        };

        assert_eq!(get("networks"), Some("arbitrum,optimism"));
        assert_eq!(get("txTypes"), Some("SWAP"));
        assert_eq!(get("protocols"), Some("uniswap"));
        assert_eq!(get("initialSearchText"), Some("USDC"));
        assert_eq!(get("sort"), Some("ASC"));
        assert_eq!(get("hideSpam"), Some("true"));
        assert_eq!(get("hideDust"), None);
        assert_eq!(get("offset"), Some("10"));
        assert_eq!(get("limit"), Some("25"));
        // The API rejects these names with 400
        assert_eq!(get("chain"), None);
        assert_eq!(get("type"), None);
    }

    #[test]
    fn test_transaction_params_defaults() {
        let params = transaction_params(&addrs(), &TransactionFilters::default(), 0, 50);
        let keys: Vec<&str> = params.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec!["addresses", "offset", "limit"]);
    }

    #[test]
    fn test_portfolio_params_default_waits_for_sync() {
        let params = portfolio_params(&PortfolioOptions::default());
        assert!(params.contains(&("waitForSync", "true".to_string())));
        assert!(params.contains(&("includeImages", "true".to_string())));

        let params = portfolio_params(&PortfolioOptions {
            no_wait_for_sync: true,
            explorer_urls: true,
        });
        assert!(!params.iter().any(|(k, _)| *k == "waitForSync"));
        assert!(params.contains(&("includeExplorerUrls", "true".to_string())));
    }

    #[test]
    fn test_api_error_message_with_validation_details() {
        let body = json!({
            "error": "Validation Failed",
            "details": { "query": [{ "message": "\"chain\" is not allowed" }] }
        });
        assert_eq!(
            api_error_message(400, &body),
            "Validation Failed: \"chain\" is not allowed"
        );
    }

    #[test]
    fn test_api_error_message_fallbacks() {
        assert_eq!(
            api_error_message(404, &json!({ "message": "Not found" })),
            "Not found"
        );
        assert_eq!(
            api_error_message(409, &json!({ "error": { "message": "Name taken" } })),
            "Name taken"
        );
        assert_eq!(
            api_error_message(500, &json!({ "message": "" })),
            "API request failed with status 500"
        );
    }
}
