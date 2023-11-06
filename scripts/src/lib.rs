use cw_orch::daemon::{ChainInfo, ChainKind, NetworkInfo};

pub mod migrate_custody;

pub const MIGALOO: NetworkInfo = NetworkInfo {
    id: "migaloo",
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
