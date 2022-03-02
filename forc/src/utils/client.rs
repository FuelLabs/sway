use anyhow::{bail, Result};
use fuel_gql_client::client::FuelClient;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Duration};

pub async fn start_fuel_core(node_url: &str, client: &FuelClient) -> Result<Child> {
    let mut url_parts = node_url.split(':').collect::<Vec<&str>>();
    let port = url_parts.pop().unwrap_or("4000");
    let ip = url_parts.join(":");

    let mut cmd = Command::new("fuel-core");
    cmd.args([format!("--port={}", port), format!("--ip={}", ip)]);
    cmd.stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(child) => {
            if client.health().await.is_ok() {
                return Ok(child);
            }

            for _ in 0..5 {
                sleep(Duration::from_millis(300)).await;
                if client.health().await.is_ok() {
                    return Ok(child);
                }
            }

            bail!("Could not start fuel-core")
        }
        Err(e) => bail!("Failed to spawn: {:?}. Error: {:?}", cmd, e),
    }
}
