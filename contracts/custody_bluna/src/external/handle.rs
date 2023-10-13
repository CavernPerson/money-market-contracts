use cosmwasm_schema::cw_serde;

#[cw_serde]

pub enum RewardContractExecuteMsg {
    /// Request bAsset reward withdrawal
    ClaimRewards { recipient: Option<String> },
}

#[cw_serde]

pub enum RewardContractQueryMsg {
    /// Request bAsset reward amount
    AccruedRewards { address: String },
}
