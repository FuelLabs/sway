use std::{fmt::Display, path::PathBuf};

/// Possible parameters to set while integrating with `fuel-core run`.
#[derive(Debug, Default)]
pub struct RunOpts {
    pub(crate) service_name: Option<String>,
    /// DB type, possible options are: `["in-memory", "rocksdb"]`.
    pub(crate) db_type: DbType,
    /// Should be used for local development only. Enabling debug mode:
    /// - Allows GraphQL Endpoints to arbitrarily advance blocks.
    /// - Enables debugger GraphQL Endpoints.
    /// - Allows setting `utxo_validation` to `false`.
    pub(crate) debug: bool,
    /// Snapshot from which to do (re)genesis.
    pub(crate) snapshot: PathBuf,
    /// Peering private key from generated key-pair.
    pub(crate) keypair: Option<String>,
    /// Ethereum RPC endpoint.
    pub(crate) relayer: Option<String>,
    /// The IP address to bind the GraphQL service to.
    pub(crate) ip: Option<std::net::IpAddr>,
    /// The port to bind the GraphQL service to.
    pub(crate) port: Option<u16>,
    /// p2p network's TCP port.
    pub(crate) peering_port: Option<u16>,
    /// The path to the database, only relevant if the db type is not
    /// "in-memory".
    pub(crate) db_path: Option<PathBuf>,
    /// Enable full utxo stateful validation.
    pub(crate) utxo_validation: bool,
    /// Use instant block production mode.
    /// Newly submitted txs will immediately trigger the production of the next block.
    pub(crate) poa_instant: bool,
    /// Enable P2P. By default, P2P is disabled.
    pub(crate) enable_p2p: bool,
    /// Addresses of the bootstrap nodes
    /// They should contain PeerId within their `Multiaddr`.
    pub(crate) bootstrap_nodes: Option<String>,
    /// The maximum number of headers to request in a single batch.
    pub(crate) sync_header_batch_size: Option<u32>,
    /// Enable the Relayer. By default, the Relayer is disabled.
    pub(crate) enable_relayer: bool,
    /// Ethereum contract address for the relayer. Requires conversion of EthAddress into fuel_types.
    pub(crate) relayer_listener: Option<String>,
    /// Number of da block that the contract is deployed at.
    pub(crate) relayer_da_deploy_height: Option<u32>,
    /// Number of pages or blocks containing logs that
    /// should be downloaded in a single call to the da layer
    pub(crate) relayer_log_page_size: Option<u32>,
    /// The maximum number of get transaction requests to make in a single batch.
    pub(crate) sync_block_stream_buffer_size: Option<u32>,
}

#[derive(Debug)]
pub enum DbType {
    InMemory,
    RocksDb,
}

impl Default for DbType {
    /// By default fuel-core interprets lack of explicit db-type declaration as
    /// db-type = rocks-db.
    fn default() -> Self {
        Self::RocksDb
    }
}

impl Display for DbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbType::InMemory => write!(f, "in-memory"),
            DbType::RocksDb => write!(f, "rocks-db"),
        }
    }
}

impl RunOpts {
    pub fn generate_params(self) -> Vec<String> {
        let mut params = vec![];
        if let Some(service_name) = self.service_name {
            params.push(format!("--service-name {service_name}"));
        }
        if self.debug {
            params.push("--debug".to_string());
        }
        if let Some(keypair) = self.keypair {
            params.push(format!("--keypair {keypair}"));
        }
        if let Some(relayer) = self.relayer {
            params.push(format!("--relayer {relayer}"));
        }
        if let Some(ip) = self.ip {
            params.push(format!("--ip {ip}"));
        }
        if let Some(port) = self.port {
            params.push(format!("--port {port}"));
        }
        if let Some(peering_port) = self.peering_port {
            params.push(format!("--peering-port {peering_port}"));
        }
        if let Some(db_path) = self.db_path {
            params.push(format!("--db-path {}", db_path.display()));
        }
        params.push(format!("--snapshot {}", self.snapshot.display()));
        params.push(format!("--db-type {}", self.db_type));
        if self.utxo_validation {
            params.push("--utxo-validation".to_string());
        }
        // --poa-instant accepts `true` or `false` as param, and it is not a
        // flag.
        if self.poa_instant {
            params.push("--poa-instant true".to_string());
        } else {
            params.push("--poa-instant false".to_string());
        }
        if self.enable_p2p {
            params.push("--enable-p2p".to_string());
        }
        if let Some(node) = self.bootstrap_nodes {
            params.push(format!("--bootstrap-nodes {node}"));
        }
        if let Some(sync_header_batch_size) = self.sync_header_batch_size {
            params.push(format!("--sync-header-batch-size {sync_header_batch_size}"));
        }
        if self.enable_relayer {
            params.push("--enable-relayer".to_string());
        }
        if let Some(relayer_listener) = self.relayer_listener {
            params.push(format!(
                "--relayer-v2-listening-contracts {relayer_listener}"
            ));
        }
        if let Some(da_deploy_height) = self.relayer_da_deploy_height {
            params.push(format!("--relayer-da-deploy-height {da_deploy_height}"));
        }
        if let Some(log_page_size) = self.relayer_log_page_size {
            params.push(format!("--relayer-log-page-size {log_page_size}"));
        }
        if let Some(sync_block) = self.sync_block_stream_buffer_size {
            params.push(format!("--sync-block-stream-buffer-size {sync_block}"));
        }
        // Split run_cmd so that each arg is actually send as a separate
        // arg. To correctly parse the args in the system level, each
        // part of an arg should go to different indices of "argv". This
        // means "--db-layer in-memory" needs to be interpreted as:
        // "--db-layer", "in-memory" to be parsed correctly.
        let params: Vec<String> = params
            .iter()
            .flat_map(|cmd| cmd.split_whitespace())
            .map(|a| a.to_string())
            .collect();
        params
    }
}
