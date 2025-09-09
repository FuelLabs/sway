use std::time::Duration;

use forc_node::local::{cmd::LocalCmd, run};
use serde_json::json;
use tokio::time::sleep;

#[ignore = "CI errors with: IO error: not a terminal"]
#[tokio::test]
async fn start_local_node_check_health() {
    let port = portpicker::pick_unused_port().expect("No ports free");
    let local_cmd = LocalCmd {
        chain_config: None,
        port: Some(port),
        db_path: None,
        account: vec![],
    };

    let _service = run(local_cmd, false).await.unwrap().unwrap();
    // Wait for node to start graphql service
    sleep(Duration::from_secs(2)).await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{port}/v1/graphql"))
        .header("Content-Type", "application/json")
        .json(&json!({
            "query": "{ health }"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    assert_eq!(body["data"]["health"], true);
}
