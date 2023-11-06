// We are migrating Whale related reward contracts so they don't include the max_spread field anymore

use cw_orch::prelude::*;
use interface::custody::LsdCustody;
use moneymarket::custody::MigrateMsg;

pub const BONE_WHALE_CUSTODY: &str =
    "migaloo1x6pdekyr3z8782k97n92hm9ldkm4h0wvlgedk9w8a5d684evq9sq3ede57";
pub const AMP_WHALE_CUSTODY: &str =
    "migaloo1eru75yglc569ezl9s7v92dhlqgqcx6e3ps2rna5vv7ywymuzdglsejfkww";

pub fn upload_custody<Chain: CwEnv>(app: Chain) -> anyhow::Result<u64> {
    let custody_contract = LsdCustody::new("custody", app.clone());
    custody_contract.upload()?;
    Ok(custody_contract.code_id()?)
}

pub fn migrate_custody<Chain: CwEnv>(
    mut app: Chain,
    sender: <Chain as TxHandler>::Sender,
) -> anyhow::Result<()> {
    // First we upload
    let code_id = upload_custody(app.clone())?;

    app.set_sender(sender);
    let custody_contract = LsdCustody::new("custody", app.clone());

    // Then we migrate (this is permissioned)
    custody_contract.set_address(&Addr::unchecked(BONE_WHALE_CUSTODY));
    custody_contract.migrate(&MigrateMsg {}, code_id)?;

    custody_contract.set_address(&Addr::unchecked(AMP_WHALE_CUSTODY));
    custody_contract.migrate(&MigrateMsg {}, code_id)?;
    Ok(())
}
