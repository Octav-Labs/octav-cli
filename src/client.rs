use reqwest::blocking::Client;
use serde_json::Value;

use crate::error::OctavError;

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
        method: &str,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<Value, OctavError> {
        let url = format!("{}{}", self.base_url, endpoint);
        eprintln!("[OCTAV] {} {}", method, url);

        let builder = match method {
            "POST" => self.client.post(&url),
            _ => self.client.get(&url),
        };

        let builder = builder
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        let builder = if let Some(b) = body {
            builder.json(&b)
        } else {
            builder
        };

        let response = builder
            .send()
            .map_err(|e| OctavError::Network(format!("Request failed: {}", e)))?;

        let status = response.status().as_u16();

        if status >= 400 {
            let text = response.text().unwrap_or_default();
            let error_data: Value = serde_json::from_str(&text)
                .unwrap_or_else(|_| serde_json::json!({ "message": text }));
            let message = error_data["message"]
                .as_str()
                .unwrap_or(&format!("API request failed with status {}", status))
                .to_string();

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

        // Handle bare number responses (e.g. /credits returns just a number)
        // and bare string responses (e.g. /sync-transactions returns a quoted string)
        serde_json::from_str(&text).map_err(|_| OctavError::Api {
            status: 200,
            message: format!("Invalid JSON response: {}", text),
        })
    }

    fn build_address_params(&self, addresses: &[String]) -> String {
        addresses
            .iter()
            .map(|a| format!("addresses={}", a))
            .collect::<Vec<_>>()
            .join("&")
    }

    // Portfolio endpoints
    pub fn get_portfolio(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request("GET", &format!("/portfolio?{}&waitForSync=true", params), None)
    }

    pub fn get_wallet(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request("GET", &format!("/wallet?{}", params), None)
    }

    pub fn get_nav(&self, addresses: &[String], currency: &str) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request(
            "GET",
            &format!("/nav?{}&currency={}", params, currency),
            None,
        )
    }

    pub fn get_token_overview(
        &self,
        addresses: &[String],
        date: &str,
    ) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request(
            "GET",
            &format!("/token-overview?{}&date={}", params, date),
            None,
        )
    }

    // Transaction endpoints
    #[allow(clippy::too_many_arguments)]
    pub fn get_transactions(
        &self,
        addresses: &[String],
        chain: Option<&str>,
        tx_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        offset: u32,
        limit: u32,
    ) -> Result<Value, OctavError> {
        let mut params = self.build_address_params(addresses);
        if let Some(c) = chain {
            params.push_str(&format!("&chain={}", c));
        }
        if let Some(t) = tx_type {
            params.push_str(&format!("&type={}", t));
        }
        if let Some(s) = start_date {
            params.push_str(&format!("&startDate={}", s));
        }
        if let Some(e) = end_date {
            params.push_str(&format!("&endDate={}", e));
        }
        params.push_str(&format!("&offset={}&limit={}", offset, limit));
        self.request("GET", &format!("/transactions?{}", params), None)
    }

    pub fn sync_transactions(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let body = serde_json::json!({ "addresses": addresses });
        self.request("POST", "/sync-transactions", Some(body))
    }

    // Historical endpoints
    pub fn get_historical(&self, addresses: &[String], date: &str) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request(
            "GET",
            &format!("/historical?{}&date={}", params, date),
            None,
        )
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
        self.request("POST", "/subscribe-snapshot", Some(body))
    }

    // Metadata endpoints
    pub fn get_status(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request("GET", &format!("/status?{}", params), None)
    }

    pub fn get_credits(&self) -> Result<Value, OctavError> {
        self.request("GET", "/credits", None)
    }

    // Specialized endpoints
    pub fn get_airdrop(&self, address: &str) -> Result<Value, OctavError> {
        self.request("GET", &format!("/airdrop?addresses={}", address), None)
    }

    pub fn get_polymarket(&self, address: &str) -> Result<Value, OctavError> {
        self.request(
            "GET",
            &format!("/portfolio/proxy/polymarket?addresses={}", address),
            None,
        )
    }

    pub fn get_agent_wallet(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request("GET", &format!("/agent/wallet?{}", params), None)
    }

    pub fn get_agent_portfolio(&self, addresses: &[String]) -> Result<Value, OctavError> {
        let params = self.build_address_params(addresses);
        self.request("GET", &format!("/agent/portfolio?{}", params), None)
    }
}
