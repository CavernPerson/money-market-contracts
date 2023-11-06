use cw_orch::{prelude::DaemonBuilder, tokio::runtime::Runtime};
use scripts::migrate_custody::upload_custody;
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

    upload_custody(app.clone())?;

    // Other functions have to be executed inside the multisig

    Ok(())
}


fn main(){
    update_global().unwrap()
}