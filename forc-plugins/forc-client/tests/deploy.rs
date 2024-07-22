use forc::cli::shared::Pkg;
use forc_client::{
    cmd,
    op::{deploy, DeployedContract},
    NodeTarget,
};
use fuel_tx::{ContractId, Salt};
use portpicker::Port;
use rexpect::spawn;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    str::FromStr,
};
use tempfile::tempdir;
use toml_edit::{Document, InlineTable, Item, Value};

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../")
        .join("../")
        .canonicalize()
        .unwrap()
}

fn test_data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test")
        .join("data")
        .canonicalize()
        .unwrap()
}

fn run_node() -> (Child, Port) {
    let port = portpicker::pick_unused_port().expect("No ports free");

    let child = Command::new("fuel-core")
        .arg("run")
        .arg("--debug")
        .arg("--db-type")
        .arg("in-memory")
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("Failed to start fuel-core");
    (child, port)
}

/// Copy a directory recursively from `source` to `dest`.
fn copy_dir(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dest)?;
    for e in fs::read_dir(source)? {
        let entry = e?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&entry.path(), &dest.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn patch_manifest_file_with_path_std(manifest_dir: &Path) -> anyhow::Result<()> {
    let toml_path = manifest_dir.join(sway_utils::constants::MANIFEST_FILE_NAME);
    let toml_content = fs::read_to_string(&toml_path).unwrap();

    let mut doc = toml_content.parse::<Document>().unwrap();
    let new_std_path = get_workspace_root().join("sway-lib-std");

    let mut std_dependency = InlineTable::new();
    std_dependency.insert("path", Value::from(new_std_path.display().to_string()));
    doc["dependencies"]["std"] = Item::Value(Value::InlineTable(std_dependency));

    fs::write(&toml_path, doc.to_string()).unwrap();
    Ok(())
}

#[tokio::test]
async fn test_simple_deploy() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let contract_ids = deploy(cmd).await.unwrap();
    node.kill().unwrap();
    let expected = vec![DeployedContract {
        id: ContractId::from_str(
            "822c8d3672471f64f14f326447793c7377b6e430122db23b622880ccbd8a33ef",
        )
        .unwrap(),
    }];

    assert_eq!(contract_ids, expected)
}

// TODO: https://github.com/FuelLabs/sway/issues/6283
// Add interactive tests for the happy path cases. This requires starting the node with funded accounts and setting up
// the wallet with the correct password. The tests should be run in a separate test suite that is not run by default.
// It would also require overriding `default_wallet_path` function for tests, so as not to interfere with the user's wallet.

#[test]
fn test_deploy_interactive_wrong_password() -> Result<(), rexpect::error::Error> {
    let (mut node, port) = run_node();
    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);

    // Spawn the forc-deploy binary using cargo run
    let project_dir = test_data_path().join("standalone_contract");
    let mut process = spawn(
        &format!(
            "cargo run --bin forc-deploy -- --node-url {node_url} -p {}",
            project_dir.display()
        ),
        Some(300000),
    )?;

    // Confirmation prompts
    process
        .exp_string("\u{1b}[1;32mConfirming\u{1b}[0m transactions [deploy standalone_contract]")?;
    process.exp_string(&format!("Network: {node_url}"))?;
    process.exp_string("Wallet: ")?;
    process.exp_string("Wallet password")?;
    process.send_line("mock_password")?;

    process.kill()?;
    node.kill().unwrap();
    Ok(())
}
