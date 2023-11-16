// We are migrating Whale related reward contracts so they don't include the max_spread field anymore

use cw_orch::prelude::*;
use cavern_moneymarket_interface::custody::LsdCustody;
use moneymarket::custody::MigrateMsg;

pub const BONE_WHALE_CUSTODY: &str =
    "terra1vmr33lncm0jhkm9gfj8824ahk50asysjgzt3ex7e94clecss8nzqftzzv2";
pub const AMP_WHALE_CUSTODY: &str =
    "terra1pggsjvjdjjr3ffm85m2kjr9ddjpnu99wts6hzxdz4cqf0kstg6gs7rnuac";
pub const B_LUNA_CUSTODY: &str = 
    "terra1sw7c9evzf44eq8k7j0kcquga0xy2ff76yhnvns4gphg37snvn26qgzjgz4";
pub const AMP_LUNA_CUSTODY: &str = 
    "terra1tlascrgjzlut6j2g4jlgv54zg3aw3c3whcjusudk24j0d3k5aucswpwzrz";


pub fn upload_custody<Chain: CwEnv>(app: Chain) -> anyhow::Result<u64> {
    let custody_contract = LsdCustody::new("custody", app.clone());
    custody_contract.upload()?;
    Ok(custody_contract.code_id()?)
}

pub fn migrate_custody<Chain: CwEnv>(
    mut app: Chain,
    admin: <Chain as TxHandler>::Sender,
    multisig: <Chain as TxHandler>::Sender,
) -> anyhow::Result<()> {
    // First we upload
    let code_id = upload_custody(app.clone())?;

    app.set_sender(admin);
    let custody_contract = LsdCustody::new("custody", app.clone());

    // Then we migrate (this is permissioned)
    custody_contract.set_address(&Addr::unchecked(BONE_WHALE_CUSTODY));
    custody_contract.migrate(&MigrateMsg {}, code_id)?;

    custody_contract.set_address(&Addr::unchecked(AMP_WHALE_CUSTODY));
    custody_contract.migrate(&MigrateMsg {}, code_id)?;


    app.set_sender(multisig);
    let custody_contract = LsdCustody::new("custody", app.clone());

    custody_contract.set_address(&Addr::unchecked(B_LUNA_CUSTODY));
    custody_contract.migrate(&MigrateMsg {}, code_id)?;

    custody_contract.set_address(&Addr::unchecked(AMP_LUNA_CUSTODY));
    custody_contract.migrate(&MigrateMsg {}, code_id)?;
    Ok(())
}
