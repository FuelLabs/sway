use crate::cli::StartCommand;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

pub fn init(command: StartCommand) -> anyhow::Result<()> {
    // If the user has a binary path they'd prefer to use, they can specify
    // it, else just use whichever indexer is in the path - whether that be
    // ia fuelup or some other means.
    let binary_path = command.bin.unwrap_or_else(|| {
        PathBuf::from(
            String::from_utf8(
                Command::new("which")
                    .arg("fuel-indexer")
                    .output()
                    .expect("❌ Failed to detect fuel-indexer binary.")
                    .stdout,
            )
            .unwrap(),
        )
    });

    let mut cmd = Command::new(&binary_path);

    if let Some(c) = &command.config {
        cmd.arg("--config").arg(c);
    }

    let mut proc = cmd
        .spawn()
        .expect("❌ Failed to spawn fuel-indexer child process.");

    // Starting the service in the background allows the user to
    // go and and continue interacting with the service (e.g., forc index deploy)
    // without having to switch terminals
    if !command.background {
        let ecode = proc
            .wait()
            .expect("❌ Failed to wait on fuel-indexer process.");

        assert!(ecode.success());
    }

    info!("\n✅ Successfully started the indexer service.");

    Ok(())
}
