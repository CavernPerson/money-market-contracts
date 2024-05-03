use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
pub const TEMP_CURRENT_TRANSFER: Item<CurrentTransfer> = Item::new("current_transfer");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub market_addr: Addr,
    pub usd_denom: String,
    pub transfer_timeout: u64,
    pub denom: String,
    pub gmp_receiver: String,
}

#[cw_serde]
pub struct CurrentTransfer {
    pub addr: String,
    pub channel_id: String,
    pub balance_before: Uint128,
}
