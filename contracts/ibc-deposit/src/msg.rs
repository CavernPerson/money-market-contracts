use cosmwasm_std::CosmosMsg;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub market_addr: String,
    pub transfer_timeout: u64,
}

pub enum ExecuteMsg {
    Deposit {
        target_channel: String,
        target_addr: String,
    },
    Withdraw {
        following_actions: CosmosMsg,
    },
}
