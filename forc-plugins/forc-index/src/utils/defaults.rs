pub const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";
pub const INDEX_LIB_FILENAME: &str = "lib.rs";
pub const DEFAULT_NAMESPACE: &str = "fuel";
pub const CARGO_CONFIG_DIR_NAME: &str = ".cargo";
pub const CARGO_CONFIG_FILENAME: &str = "config";
pub const DEFAULT_INDEXER_URL: &str = "http://127.0.0.1:29987";

pub fn default_index_cargo_toml(index_name: &str) -> String {
    format!(
        r#"[package]
name = "{index_name}"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ['cdylib']

[dependencies]
fuel-indexer-macros = {{ version = "0.1", default-features = false }}
fuel-indexer-plugin = {{ version = "0.1" }}
fuel-indexer-schema = {{ version = "0.1", default-features = false }}
fuel-tx = "0.23"
fuels-core = "0.30"
fuels-types = "0.30"
getrandom = {{ version = "0.2", features = ["js"] }}
serde = {{ version = "1.0", default-features = false, features = ["derive"] }}
"#
    )
}

pub fn default_index_manifest(namespace: &str, index_name: &str, project_path: &str) -> String {
    format!(
        r#"namespace: {namespace}
identifier: {index_name}
# abi: /path/to/your/contract-abi.json
graphql_schema: {project_path}/schema/{index_name}.schema.graphql
module:
  wasm: /path/to/your/index_wasm_module.wasm
"#
    )
}

pub fn default_index_lib(index_name: &str, manifest_filename: &str, path: &str) -> String {
    format!(
        r#"extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(manifest = "{path}/{manifest_filename}")]
pub mod {index_name}_index_mod {{

    fn {index_name}_handler(block: BlockData) {{
        Logger::info("Processing a block. (>'.')>");
        for tx in block.transactions.iter() {{
            let msg = format!("Processing transaction: {{}}", tx.id);
            Logger::info(&msg);
        }}
    }}
}}
"#
    )
}

pub fn default_index_schema() -> String {
    r#"schema {
    query: QueryRoot
}

type QueryRoot {
    account: Account
}

type Account {
    id: ID!
    address: Address! @indexed
    first_seen: UInt8!
    last_seen: UInt8!
}

"#
    .to_string()
}

pub fn default_cargo_config() -> String {
    r#"[build]
target = "wasm32-unknown-unknown"
"#
    .to_string()
}
