use cosmwasm_schema::cw_serde;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub market_addr: String,
    pub transfer_timeout: u64,
    pub gmp_receiver: String,
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
    Withdraw {
        to_axelar_channel: String,
        axelar_receiver: String,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {
    pub gmp_receiver: String,
}

#[cw_serde]
pub struct SendTokensGmpMessage {
    pub destination_chain: String,
    pub destination_address: String,
    #[serde(rename = "type")]
    pub type_: i64,
}
