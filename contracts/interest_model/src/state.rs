use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CanonicalAddr, Decimal256, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

static KEY_CONFIG: &[u8] = b"config";

#[cw_serde]
pub struct OldConfig {
    pub owner: CanonicalAddr,
    pub base_rate: Decimal256,
    pub interest_multiplier: Decimal256,
}

#[cw_serde]
pub struct Config {
    pub owner: CanonicalAddr,
    pub base_rate: Decimal256,
    pub first_interest_multiplier: Decimal256,
    pub target_utilization_rate: Decimal256,
    pub second_interest_multiplier: Decimal256,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn read_old_config(storage: &dyn Storage) -> StdResult<OldConfig> {
    singleton_read(storage, KEY_CONFIG).load()
}
