use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

static KEY_CONFIG: &[u8] = b"config";

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub sender_contract: Addr,
    pub overseer_contract: Addr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
