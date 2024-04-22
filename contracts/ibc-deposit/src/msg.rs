use cosmwasm_std::CosmosMsg;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub market_addr: String,
    pub transfer_timeout: u64,
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    #[payable]
    Deposit {
        target_channel: String,
        target_addr: String,
    },
    #[payable]
    Withdraw { following_actions: CosmosMsg },
}
