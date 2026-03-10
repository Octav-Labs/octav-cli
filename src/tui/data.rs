#![allow(dead_code)]
use std::collections::HashMap;

use serde_json::Value;

pub struct NavData {
    pub nav: f64,
    pub currency: String,
}

pub struct ChainAllocation {
    pub chain_name: String,
    pub chain_key: String,
    pub value: f64,
    pub percentage: f64,
}

pub struct HoldingRow {
    pub symbol: String,
    pub name: String,
    pub image_url: String,
    pub chain: String,
    pub chain_key: String,
    pub protocol: String,
    pub protocol_key: String,
    pub balance: f64,
    pub price: f64,
    pub value: f64,
    pub percentage: f64,
}

pub struct ProtocolData {
    pub name: String,
    pub key: String,
    pub image_url: String,
    pub total_value: f64,
    pub chains: Vec<ProtocolChainData>,
}

pub struct ProtocolChainData {
    pub name: String,
    pub key: String,
    pub value: f64,
    pub positions: Vec<PositionData>,
}

pub struct PositionData {
    pub name: String,
    pub total_value: f64,
    pub assets: Vec<PositionAsset>,
}

pub struct PositionAsset {
    pub symbol: String,
    pub name: String,
    pub image_url: String,
    pub asset_type: String,
    pub balance: f64,
    pub price: f64,
    pub value: f64,
}

pub struct TransactionItem {
    pub hash: String,
    pub timestamp: String,
    pub date_display: String,
    pub chain_name: String,
    pub protocol_name: String,
    pub tx_type: String,
    pub assets_in: Vec<TxAsset>,
    pub assets_out: Vec<TxAsset>,
    pub fees_fiat: f64,
}

pub struct TxAsset {
    pub symbol: String,
    pub balance: f64,
}

fn parse_f64(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn num_field(v: &Value, key: &str) -> f64 {
    v.get(key).map(|v| parse_f64(v)).unwrap_or(0.0)
}

pub fn parse_nav(json: &Value) -> NavData {
    NavData {
        nav: num_field(json, "nav"),
        currency: str_field(json, "currency"),
    }
}

pub fn parse_credits(json: &Value) -> f64 {
    parse_f64(json)
}

/// Extract assets from a sub-position object.
/// The API puts assets in typed arrays: supplyAssets, borrowAssets, rewardAssets, dexAssets, etc.
/// Also check the top-level `assets` array.
fn collect_assets_from_sub_position(sub_pos: &Value) -> Vec<(PositionAsset, String)> {
    let asset_arrays = [
        ("assets", ""),
        ("supplyAssets", "Supplied"),
        ("borrowAssets", "Borrowed"),
        ("rewardAssets", "Reward"),
        ("dexAssets", "DEX"),
        ("marginAssets", "Margin"),
        ("baseAssets", "Base"),
        ("quoteAssets", "Quote"),
        ("collateralizeNFTAssets", "Collateral"),
    ];

    let mut results = Vec::new();

    for (field, asset_type) in &asset_arrays {
        if let Some(arr) = sub_pos.get(field).and_then(|v| v.as_array()) {
            for asset in arr {
                let balance = num_field(asset, "balance");
                let value = num_field(asset, "value");
                let symbol = str_field(asset, "symbol");

                // Skip zero-value/zero-balance assets with no symbol
                if symbol.is_empty() && balance == 0.0 && value == 0.0 {
                    continue;
                }

                results.push((
                    PositionAsset {
                        symbol: symbol.clone(),
                        name: str_field(asset, "name"),
                        image_url: str_field(asset, "imgSmall"),
                        asset_type: asset_type.to_string(),
                        balance,
                        price: num_field(asset, "price"),
                        value,
                    },
                    symbol,
                ));
            }
        }
    }

    results
}

pub fn parse_portfolio(
    json: &Value,
) -> (Vec<ProtocolData>, Vec<HoldingRow>, Vec<ChainAllocation>) {
    let mut protocols: Vec<ProtocolData> = Vec::new();
    let mut all_holdings: Vec<HoldingRow> = Vec::new();
    let mut chain_values: HashMap<String, (String, f64, f64)> = HashMap::new(); // key -> (name, value, percentage)

    let empty_arr = Vec::new();
    let entries = json.as_array().unwrap_or(&empty_arr);

    for entry in entries {
        // Extract chain allocations from root-level `chains` object
        if let Some(chains_obj) = entry.get("chains").and_then(|v| v.as_object()) {
            for (chain_key, chain_val) in chains_obj {
                let value = num_field(chain_val, "value");
                let pct = num_field(chain_val, "valuePercentile");
                let name = str_field(chain_val, "name");
                if value > 0.0 {
                    let e = chain_values
                        .entry(chain_key.clone())
                        .or_insert((name, 0.0, 0.0));
                    e.1 += value;
                    e.2 = pct; // Use the API-provided percentage
                }
            }
        }

        // Parse protocols from assetByProtocols
        let asset_by_protocols = match entry.get("assetByProtocols") {
            Some(v) if v.is_object() => v,
            _ => continue,
        };

        let proto_map = asset_by_protocols.as_object().unwrap();

        for (proto_key, proto_val) in proto_map {
            let proto_name = str_field(proto_val, "name");
            let proto_name = if proto_name.is_empty() {
                proto_key.clone()
            } else {
                proto_name
            };
            let proto_image = str_field(proto_val, "imgSmall");
            let proto_total = num_field(proto_val, "value");

            // Skip zero-value protocols
            if proto_total == 0.0 {
                continue;
            }

            let mut proto_chains: Vec<ProtocolChainData> = Vec::new();

            if let Some(chains) = proto_val.get("chains").and_then(|v| v.as_object()) {
                for (chain_key, chain_val) in chains {
                    let chain_name = str_field(chain_val, "name");
                    let chain_name = if chain_name.is_empty() {
                        chain_key.clone()
                    } else {
                        chain_name
                    };
                    let chain_value = num_field(chain_val, "value");

                    // Skip zero-value chains
                    if chain_value == 0.0 {
                        continue;
                    }

                    let mut positions: Vec<PositionData> = Vec::new();

                    if let Some(pos_map) =
                        chain_val.get("protocolPositions").and_then(|v| v.as_object())
                    {
                        for (_pos_type_key, pos_type_val) in pos_map {
                            let pos_type_name = str_field(pos_type_val, "name");
                            let pos_type_total = num_field(pos_type_val, "totalValue");

                            // Check for nested protocolPositions array (DeFi protocols)
                            if let Some(sub_positions) = pos_type_val
                                .get("protocolPositions")
                                .and_then(|v| v.as_array())
                            {
                                if !sub_positions.is_empty() {
                                    // Each sub-position is a separate position
                                    for sub_pos in sub_positions {
                                        let sub_name = str_field(sub_pos, "name");
                                        let sub_value = num_field(sub_pos, "value");

                                        if sub_value == 0.0 {
                                            continue;
                                        }

                                        let asset_pairs =
                                            collect_assets_from_sub_position(sub_pos);
                                        let assets: Vec<PositionAsset> =
                                            asset_pairs.iter().map(|(a, _)| {
                                                PositionAsset {
                                                    symbol: a.symbol.clone(),
                                                    name: a.name.clone(),
                                                    image_url: a.image_url.clone(),
                                                    asset_type: a.asset_type.clone(),
                                                    balance: a.balance,
                                                    price: a.price,
                                                    value: a.value,
                                                }
                                            }).collect();

                                        // Add to holdings
                                        for asset in &assets {
                                            if asset.value.abs() > 0.001 || asset.balance.abs() > 0.0 {
                                                all_holdings.push(HoldingRow {
                                                    symbol: asset.symbol.clone(),
                                                    name: asset.name.clone(),
                                                    image_url: asset.image_url.clone(),
                                                    chain: chain_name.clone(),
                                                    chain_key: chain_key.clone(),
                                                    protocol: proto_name.clone(),
                                                    protocol_key: proto_key.clone(),
                                                    balance: asset.balance,
                                                    price: asset.price,
                                                    value: asset.value,
                                                    percentage: 0.0,
                                                });
                                            }
                                        }

                                        positions.push(PositionData {
                                            name: sub_name,
                                            total_value: sub_value,
                                            assets,
                                        });
                                    }
                                    continue;
                                }
                            }

                            // Direct assets (e.g., wallet protocol)
                            if let Some(asset_arr) =
                                pos_type_val.get("assets").and_then(|v| v.as_array())
                            {
                                if !asset_arr.is_empty() {
                                    let mut assets: Vec<PositionAsset> = Vec::new();

                                    for asset in asset_arr {
                                        let symbol = str_field(asset, "symbol");
                                        let balance = num_field(asset, "balance");
                                        let value = num_field(asset, "value");
                                        let price = num_field(asset, "price");

                                        if value.abs() < 0.001 && balance == 0.0 {
                                            continue;
                                        }

                                        all_holdings.push(HoldingRow {
                                            symbol: symbol.clone(),
                                            name: str_field(asset, "name"),
                                            image_url: str_field(asset, "imgSmall"),
                                            chain: chain_name.clone(),
                                            chain_key: chain_key.clone(),
                                            protocol: proto_name.clone(),
                                            protocol_key: proto_key.clone(),
                                            balance,
                                            price,
                                            value,
                                            percentage: 0.0,
                                        });

                                        assets.push(PositionAsset {
                                            symbol,
                                            name: str_field(asset, "name"),
                                            image_url: str_field(asset, "imgSmall"),
                                            asset_type: String::new(),
                                            balance,
                                            price,
                                            value,
                                        });
                                    }

                                    if !assets.is_empty() {
                                        positions.push(PositionData {
                                            name: pos_type_name.clone(),
                                            total_value: pos_type_total,
                                            assets,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    if !positions.is_empty() {
                        proto_chains.push(ProtocolChainData {
                            name: chain_name,
                            key: chain_key.clone(),
                            value: chain_value,
                            positions,
                        });
                    }
                }
            }

            // Merge if protocol already exists (multi-address)
            if let Some(existing) = protocols.iter_mut().find(|p| p.key == *proto_key) {
                existing.total_value += proto_total;
                for new_chain in proto_chains {
                    if let Some(existing_chain) =
                        existing.chains.iter_mut().find(|c| c.key == new_chain.key)
                    {
                        existing_chain.value += new_chain.value;
                        existing_chain.positions.extend(new_chain.positions);
                    } else {
                        existing.chains.push(new_chain);
                    }
                }
            } else if !proto_chains.is_empty() {
                protocols.push(ProtocolData {
                    name: proto_name,
                    key: proto_key.clone(),
                    image_url: proto_image,
                    total_value: proto_total,
                    chains: proto_chains,
                });
            }
        }
    }

    // Sort protocols by value descending
    protocols.sort_by(|a, b| {
        b.total_value
            .partial_cmp(&a.total_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Sort holdings by value descending
    all_holdings.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Calculate holding percentages
    let total_value: f64 = all_holdings.iter().map(|h| h.value).sum();
    if total_value > 0.0 {
        for h in &mut all_holdings {
            h.percentage = (h.value / total_value) * 100.0;
        }
    }

    // Build chain allocations from collected values
    let mut chain_allocs: Vec<ChainAllocation> = chain_values
        .into_iter()
        .map(|(key, (name, value, percentage))| ChainAllocation {
            chain_name: name,
            chain_key: key,
            value,
            percentage,
        })
        .collect();
    chain_allocs.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    (protocols, all_holdings, chain_allocs)
}

pub fn parse_transactions(json: &Value) -> (Vec<TransactionItem>, u64) {
    let empty_arr = Vec::new();
    let tx_array = json
        .get("transactions")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_arr);

    let total = tx_array.len() as u64;

    let items: Vec<TransactionItem> = tx_array
        .iter()
        .map(|tx| {
            // Timestamp is Unix seconds as string
            let timestamp = str_field(tx, "timestamp");
            let date_display = format_unix_timestamp(&timestamp);

            // Chain is an object: { key, name, ... }
            let chain_name = tx
                .get("chain")
                .and_then(|c| c.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let protocol_name = tx
                .get("protocol")
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let assets_in = parse_tx_assets(tx.get("assetsIn"));
            let assets_out = parse_tx_assets(tx.get("assetsOut"));

            TransactionItem {
                hash: str_field(tx, "hash"),
                timestamp: timestamp.clone(),
                date_display,
                chain_name,
                protocol_name,
                tx_type: str_field(tx, "type"),
                assets_in,
                assets_out,
                fees_fiat: num_field(tx, "feesFiat"),
            }
        })
        .collect();

    (items, total)
}

fn parse_tx_assets(v: Option<&Value>) -> Vec<TxAsset> {
    let arr = match v.and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return Vec::new(),
    };

    arr.iter()
        .map(|a| TxAsset {
            symbol: str_field(a, "symbol"),
            balance: num_field(a, "balance"),
        })
        .collect()
}

fn format_unix_timestamp(ts: &str) -> String {
    let secs: i64 = match ts.parse() {
        Ok(s) => s,
        Err(_) => return ts.to_string(),
    };

    // Manual date calculation from Unix timestamp
    let days = secs / 86400;
    let remaining = secs % 86400;
    let _hours = remaining / 3600;

    // Days since epoch (1970-01-01) to year/month/day
    let (year, month, day) = days_to_date(days);

    let month_name = match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    };

    format!("{} {:02} '{}", month_name, day, year % 100)
}

fn days_to_date(days_since_epoch: i64) -> (i64, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
