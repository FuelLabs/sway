use super::cmd::TestnetCmd;
use crate::{
    cmd::{ask_user_discreetly, ask_user_string, ask_user_yes_no_question},
    consts::{
        TESTNET_RELAYER_DA_DEPLOY_HEIGHT, TESTNET_RELAYER_LISTENING_CONTRACT,
        TESTNET_RELAYER_LOG_PAGE_SIZE, TESTNET_RESERVED_NODE, TESTNET_SERVICE_NAME,
        TESTNET_SYNC_BLOCK_STREAM_BUFFER_SIZE, TESTNET_SYNC_HEADER_BATCH_SIZE,
    },
    op::HumanReadableCommand,
    pkg::{create_chainconfig_dir, ChainConfig},
    run_opts::{DbType, RunOpts},
};
use anyhow::Context;
use forc_tracing::println_green;
use forc_util::forc_result_bail;
use serde::{Deserialize, Serialize};
use std::{
    net::IpAddr,
    path::PathBuf,
    process::{Child, Command},
};

/// Configures the node with testnet configuration to connect the node to latest testnet.
/// Returns `None` if this is a dry_run and no child process created for fuel-core.
pub(crate) fn run(cmd: TestnetCmd, dry_run: bool) -> anyhow::Result<Option<Child>> {
    create_chainconfig_dir(ChainConfig::Testnet)?;
    let (peer_id, secret) = if let (Some(peer_id), Some(secret)) = (&cmd.peer_id, &cmd.secret) {
        (peer_id.clone(), secret.clone())
    } else {
        let has_keypair = ask_user_yes_no_question("Do you have a keypair in hand?")?;
        if has_keypair {
            // ask the keypair
            let peer_id = ask_user_string("Peer Id:")?;
            let secret = ask_user_discreetly("Secret:")?;
            (peer_id, secret)
        } else {
            forc_result_bail!(
                "Please create a keypair with `fuel-core-keygen new --key-type peering`"
            );
        }
    };

    let relayer = if let Some(relayer) = cmd.relayer {
        relayer
    } else {
        ask_user_string("Ethereum RPC (Sepolia) Endpoint:")?
    };

    let keypair = KeyPair { peer_id, secret };

    let opts = TestnetOpts {
        keypair,
        relayer,
        ip: cmd.ip,
        port: cmd.port,
        peering_port: cmd.peering_port,
        db_path: cmd.db_path,
    };
    let run_opts = RunOpts::from(opts);
    let params = run_opts.generate_params();
    let mut fuel_core_command = Command::new("fuel-core");
    fuel_core_command.arg("run");
    fuel_core_command.args(params.as_slice());
    if dry_run {
        println_green(&format!(
            "{}",
            HumanReadableCommand::from(fuel_core_command)
        ));
        Ok(None)
    } else {
        // Spawn the process with proper error handling
        let handle = fuel_core_command
            .spawn()
            .with_context(|| "Failed to spawn fuel-core process:".to_string())?;
        Ok(Some(handle))
    }
}

#[derive(Debug)]
pub struct TestnetOpts {
    keypair: KeyPair,
    relayer: String,
    ip: IpAddr,
    port: u16,
    peering_port: u16,
    db_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    peer_id: String,
    secret: String,
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
            utxo_validation: true,
            poa_instant: false,
            enable_p2p: true,
            reserved_nodes: Some(TESTNET_RESERVED_NODE.to_string()),
            sync_header_batch_size: Some(TESTNET_SYNC_HEADER_BATCH_SIZE),
            enable_relayer: true,
            relayer_listener: Some(TESTNET_RELAYER_LISTENING_CONTRACT.to_string()),
            relayer_da_deploy_height: Some(TESTNET_RELAYER_DA_DEPLOY_HEIGHT),
            relayer_log_page_size: Some(TESTNET_RELAYER_LOG_PAGE_SIZE),
            sync_block_stream_buffer_size: Some(TESTNET_SYNC_BLOCK_STREAM_BUFFER_SIZE),
            bootstrap_nodes: None,
        }
    }
}
