use cw_orch::{daemon::{ChainInfo, ChainKind, NetworkInfo}, prelude::{ContractInstance}, networks::MIGALOO_1};
use interface::{overseer::Overseer};
use cw_orch::prelude::*;

fn update_global_test() -> anyhow::Result<()>{

    // First we create a fork testing object
    pretty_env_logger::init();

    let bone_whale_hub = "migaloo196slekmpf56972v6456lurvuma92pq265gs700zztkx2j83zy7xstmmqy8";
    let amp_whale_hub = "migaloo16qlww0z979rjhzrstj4kwxftgx5fqkup6cxvln9xg9jk7w72638q4tgknu";
    let amp_luna_hub = "migaloo1sv2zgqr5u4lwns80k07a8m9vfyf65hq64fklhxqwg8dslv2nqpwsddjkel";
    let b_luna_hub = "migaloo1kw7ga0w4c3hfskc6zc2pdc0x6qwvznrstvewzn6w5chsc249sl6sgqzwsw";
    let amp_roar_hub = "migaloo1exx0zzl003kgva0f5qmwdelnse98am5mrshp52w9vy8zqdpk8d3q8xq9py";

    let bone_whale_token = "migaloo1rlsyqe67ptr60ljcrhrrylh27u4ay2yspqrkazw2zyg0p4snfamqvnfchp";
    let amp_whale_token = "migaloo1xg4uskjek0r9tgcwu3xzehxvfcyq25236z6rqmxjnm70mulwhazq39vlph";
    let amp_luna_token = "migaloo1vxk46cng9jzn9ztv25r0sjq4s8r96scwuzdwzgpmedple4tvnynsumglrh";
    let b_luna_token = "migaloo1ssvcrf0p0hds8rfsxalgk97qe8t4zcumhaq2hm8pqdk29r9vy6uqluw4wc";
    let amp_roar_token = "migaloo1hztttrjkpq3zsp66qxajgzrvhqgyjgqsn35h76xs7602jgue8yuqa3q5lz";

    let bone_whale_reward = "migaloo1m5yx7zdtdx6q8qd6njazu3uyk0drtcxhkh9cydym85ss5ktdppcswcztt9";
    let amp_whale_reward = "migaloo1fjf4rnt9p2xa4uu73tsczve06stg8egvnhlg6nxxntfgc3pfv7jqd6hq4w";
    let amp_luna_reward = "migaloo19ph6tjl56rrpqhfqy9ura42qlczdc3r72flu850mum4lmpql4lmqz8l2ef";
    let b_luna_reward = "migaloo1zy4nh9xyc6u3lhnm6gsjm6lkvs23hwgkul9ntksudewudkrt57jqqzghag";
    let amp_roar_reward = "migaloo1y6ctlfsvgnldgrmwsp0umhyep4yqksrlyk7tq09jtlz4u33z39gs4846mu";

    let multisig = "migaloo1epn5jukddvlf5v3k6f7cwqc90kgm9ew88wp9jm";

    let overseer_addr = "migaloo18qatlena5eujecsuwrwkpr5qccjddrf8ss4ykzlx8gmrt5dlxxkqysf3lz";

    // We migrate
    let app = cw_orch_fork_mock::ForkMock::new(MIGALOO_1);

    // We execute epoch operations on the overseer contract
    let overseer = Overseer::new("overseer", app.clone());
    overseer.set_address(overseer_addr);


    overseer.execute_epoch_operations();


    let sender = Addr::unchecked(multisig);
    migrate_rewards(app.clone(), sender.clone())?;
    migrate_hub_and_change_reward(app.clone(), sender)?;

    update_global(app.clone(), bone_whale_hub.to_string())?;
    update_global(app.clone(), amp_whale_hub.to_string())?;
    update_global(app.clone(), amp_luna_hub.to_string())?;
    update_global(app.clone(), b_luna_hub.to_string())?;
    update_global(app.clone(), amp_roar_hub.to_string())?;

    let analysis = app.storage_analysis();

    analysis.compare_all_readable_contract_storage();
    analysis.compare_all_balances();

    Ok(())
}

fn update_global(app: cw_orch_fork_mock::ForkMock, hub: String) -> anyhow::Result<()>{

    let hub_contract = LsdHub::new("hub", app.clone());
    hub_contract.set_address(&Addr::unchecked(hub));

    // We try to update the global index
    hub_contract.update_global_index()?;

    Ok(())
}

fn main(){
    // update_global_test().unwrap()
}