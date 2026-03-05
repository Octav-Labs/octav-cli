use serde_json::Value;

const ROOT_KEYS: &[&str] = &[
    "closedPnl",
    "dailyIncome",
    "dailyExpense",
    "fees",
    "feesFiat",
    "openPnl",
    "manualBalanceNetworth",
    "totalCostBasis",
];

const NESTED_KEYS: &[&str] = &["totalCostBasis", "totalClosedPnl", "totalOpenPnl"];

const ASSET_KEYS: &[&str] = &["openPnl", "totalCostBasis"];

fn strip_keys(obj: &mut serde_json::Map<String, Value>, keys: &[&str]) {
    for &key in keys {
        obj.remove(key);
    }
    obj.remove("uuid");
}

fn strip_assets(assets: &mut Value) {
    if let Some(arr) = assets.as_array_mut() {
        for asset in arr {
            if let Some(obj) = asset.as_object_mut() {
                for &key in ASSET_KEYS {
                    obj.remove(key);
                }
                obj.remove("uuid");
            }
        }
    }
}

fn strip_protocol_position(position: &mut serde_json::Map<String, Value>) {
    strip_keys(position, NESTED_KEYS);
    if let Some(assets) = position.get_mut("assets") {
        strip_assets(assets);
    }
}

fn strip_chain(chain: &mut serde_json::Map<String, Value>) {
    strip_keys(chain, NESTED_KEYS);
    if let Some(positions) = chain.get_mut("protocolPositions") {
        if let Some(pos_map) = positions.as_object_mut() {
            for pos in pos_map.values_mut() {
                if let Some(obj) = pos.as_object_mut() {
                    strip_protocol_position(obj);
                }
            }
        }
    }
}

fn strip_protocol(protocol: &mut serde_json::Map<String, Value>) {
    strip_keys(protocol, NESTED_KEYS);
    if let Some(chains) = protocol.get_mut("chains") {
        if let Some(chains_map) = chains.as_object_mut() {
            for chain in chains_map.values_mut() {
                if let Some(obj) = chain.as_object_mut() {
                    strip_chain(obj);
                }
            }
        }
    }
}

pub fn strip_portfolio_fields(data: &mut Value) {
    if let Some(arr) = data.as_array_mut() {
        for entry in arr {
            if let Some(obj) = entry.as_object_mut() {
                strip_keys(obj, ROOT_KEYS);
                if let Some(protocols) = obj.get_mut("assetByProtocols") {
                    if let Some(proto_map) = protocols.as_object_mut() {
                        for protocol in proto_map.values_mut() {
                            if let Some(proto_obj) = protocol.as_object_mut() {
                                strip_protocol(proto_obj);
                            }
                        }
                    }
                }
            }
        }
    }
}
