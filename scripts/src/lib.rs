use cw_orch::environment::{ChainInfo, ChainKind, NetworkInfo};

pub mod migrate_custody;

pub const MIGALOO: NetworkInfo = NetworkInfo {
    chain_name: "migaloo",
    pub_address_prefix: "migaloo",
    coin_type: 118,
};

pub const MIGALOO_1: ChainInfo = ChainInfo {
    chain_id: "migaloo-1",
    gas_denom: "uwhale",
    gas_price: 1f64,
    grpc_urls: &["http://migaloo-grpc.polkachu.com:20790"],
    lcd_url: None,
    fcd_url: None,
    network_info: MIGALOO,
    kind: ChainKind::Mainnet,
};

pub const AXELAR: NetworkInfo = NetworkInfo {
    chain_name: "axelar",
    pub_address_prefix: "axelar",
    coin_type: 118,
};

pub const AXELAR_1: ChainInfo = ChainInfo {
    chain_id: "axelar-dojo-1",
    gas_denom: "uaxl",
    gas_price: 0.007,
    grpc_urls: &["http://axelar-grpc.polkachu.com:15190"],
    lcd_url: None,
    fcd_url: None,
    network_info: AXELAR,
    kind: ChainKind::Mainnet,
};
