use crate::{
    chain_config::{check_and_update_chain_config, ChainConfig},
    consts::{
        TESTNET_RELAYER_DA_DEPLOY_HEIGHT, TESTNET_RELAYER_LISTENING_CONTRACT,
        TESTNET_RELAYER_LOG_PAGE_SIZE, TESTNET_SERVICE_NAME, TESTNET_SYNC_BLOCK_STREAM_BUFFER_SIZE,
        TESTNET_SYNC_HEADER_BATCH_SIZE,
    },
    run_opts::{DbType, RunOpts},
    testnet::cmd::TestnetCmd,
    util::{ask_user_keypair, ask_user_string, HumanReadableCommand, KeyPair},
};
use anyhow::Context;
use forc_diagnostic::println_green;
use std::{
    net::IpAddr,
    path::PathBuf,
    process::{Child, Command},
};

/// Configures the node with testnet configuration to connect the node to latest testnet.
/// Returns `None` if this is a dry_run and no child process created for fuel-core.
pub async fn run(cmd: TestnetCmd, dry_run: bool) -> anyhow::Result<Option<Child>> {
    check_and_update_chain_config(ChainConfig::Testnet).await?;
    let keypair = if let (Some(peer_id), Some(secret)) = (
        &cmd.connection_settings.peer_id,
        &cmd.connection_settings.secret,
    ) {
        KeyPair {
            peer_id: peer_id.clone(),
            secret: secret.clone(),
        }
    } else {
        ask_user_keypair()?
    };

    let relayer = cmd.connection_settings.relayer.unwrap_or_else(|| {
        ask_user_string("Ethereum RPC (Sepolia) Endpoint:").expect("Failed to get RPC endpoint")
    });

    let opts = TestnetOpts {
        keypair,
        relayer,
        ip: cmd.connection_settings.ip,
        port: cmd.connection_settings.port,
        peering_port: cmd.connection_settings.peering_port,
        db_path: cmd.db_path,
        bootstrap_node: cmd.bootstrap_node,
    };
    let run_opts = RunOpts::from(opts);
    let params = run_opts.generate_params();
    let mut fuel_core_command = Command::new("fuel-core");
    fuel_core_command.arg("run");
    fuel_core_command.args(params.as_slice());

    println_green(&format!(
        "{}",
        HumanReadableCommand::from(&fuel_core_command)
    ));

    if dry_run {
        return Ok(None);
    }

    // Spawn the process with proper error handling
    let handle = fuel_core_command
        .spawn()
        .with_context(|| "Failed to spawn fuel-core process:".to_string())?;
    Ok(Some(handle))
}

#[derive(Debug)]
pub struct TestnetOpts {
    keypair: KeyPair,
    relayer: String,
    ip: IpAddr,
    port: u16,
    peering_port: u16,
    db_path: PathBuf,
    bootstrap_node: String,
}

impl From<TestnetOpts> for RunOpts {
    fn from(value: TestnetOpts) -> Self {
        Self {
            service_name: Some(TESTNET_SERVICE_NAME.to_string()),
            db_type: DbType::RocksDb,
            debug: false,
            snapshot: ChainConfig::Testnet.into(),
            keypair: Some(value.keypair.secret),
            relayer: Some(value.relayer),
            ip: Some(value.ip),
            port: Some(value.port),
            peering_port: Some(value.peering_port),
            db_path: Some(value.db_path),
            bootstrap_nodes: Some(value.bootstrap_node),
            utxo_validation: true,
            poa_instant: false,
            enable_p2p: true,
            sync_header_batch_size: Some(TESTNET_SYNC_HEADER_BATCH_SIZE),
            enable_relayer: true,
            relayer_listener: Some(TESTNET_RELAYER_LISTENING_CONTRACT.to_string()),
            relayer_da_deploy_height: Some(TESTNET_RELAYER_DA_DEPLOY_HEIGHT),
            relayer_log_page_size: Some(TESTNET_RELAYER_LOG_PAGE_SIZE),
            sync_block_stream_buffer_size: Some(TESTNET_SYNC_BLOCK_STREAM_BUFFER_SIZE),
        }
    }
}
