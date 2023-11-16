use cavern_lsd_wrapper_interface::LsdWrapperWithLimit;
use cosmwasm_std::{Addr, Empty};
use cw_orch::prelude::{ContractInstance, CwOrchUpload, CwOrchMigrate};
use cw_orch::{prelude::DaemonBuilder, tokio::runtime::Runtime};
use scripts::MIGALOO_1;


fn update_global() -> anyhow::Result<()>{

    dotenv::dotenv()?;
    pretty_env_logger::init();

    // We upload code_ids
    let rt = Runtime::new()?;
    let app = DaemonBuilder::default()
        .chain(MIGALOO_1)
        .handle(rt.handle())
        .build()?;
    
    let amp_whalet_wrapper_addr = "migaloo187ye4suzmdt8q0p330jae97m892h6v4gkj9sp4uuxpdjatv36n8qt43alj";
    let wrapper_token = LsdWrapperWithLimit::new("ampWhalet::token_wrapper", app.clone());
    wrapper_token.set_address(&Addr::unchecked(amp_whalet_wrapper_addr));
    
    // We need to migrate the token contract to the corrected version
    wrapper_token.upload()?;
    wrapper_token.migrate(&Empty{}, wrapper_token.code_id()?)?;


    // Other functions have to be executed inside the multisig

    Ok(())
}


fn main(){
    update_global().unwrap()
}