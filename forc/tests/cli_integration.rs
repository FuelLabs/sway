use std::path::PathBuf;

use rexpect::spawn;

const TIMEOUT_MS: u64 = 300000;

fn test_fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .canonicalize()
        .unwrap()
}

#[test]
fn test_forc_test_decoded_logs() -> Result<(), rexpect::error::Error> {
    // Spawn the forc binary using cargo run
    let project_dir = test_fixtures_path().join("test_contract");
    let mut process = spawn(
        &format!(
            "cargo run --bin forc -- test --logs --path {}",
            project_dir.to_string_lossy()
        ),
        Some(TIMEOUT_MS),
    )?;

    // Assert that the output is correct
    process.exp_string("      test test_log_4")?;
    process.exp_string("decoded log values:")?;
    process.exp_string("4, log rb: 1515152261580153489")?;
    process.exp_string("      test test_log_2")?;
    process.exp_string("decoded log values:")?;
    process.exp_string("2, log rb: 1515152261580153489")?;

    process.process.exit()?;
    Ok(())
}

#[test]
fn test_forc_test_raw_logs() -> Result<(), rexpect::error::Error> {
    // Spawn the forc binary using cargo run
    let project_dir = test_fixtures_path().join("test_contract");
    let mut process = spawn(
        &format!(
            "cargo run --bin forc -- test --raw-logs --path {}",
            project_dir.to_string_lossy()
        ),
        Some(TIMEOUT_MS),
    )?;

    // Assert that the output is correct
    process.exp_string("      test test_log_4")?;
    process.exp_string("raw logs:")?;
    process.exp_string(r#"[{"LogData":{"data":"0000000000000004","digest":"8005f02d43fa06e7d0585fb64c961d57e318b27a145c857bcd3a6bdb413ff7fc","id":"0000000000000000000000000000000000000000000000000000000000000000","is":10368,"len":8,"pc":11292,"ptr":12480,"ra":0,"rb":1515152261580153489}}]"#)?;
    process.exp_string("      test test_log_2")?;
    process.exp_string("raw logs:")?;
    process.exp_string(r#"[{"LogData":{"data":"0000000000000002","digest":"cd04a4754498e06db5a13c5f371f1f04ff6d2470f24aa9bd886540e5dce77f70","id":"0000000000000000000000000000000000000000000000000000000000000000","is":10368,"len":8,"pc":11292,"ptr":12480,"ra":0,"rb":1515152261580153489}}]"#)?;

    process.process.exit()?;
    Ok(())
}

#[test]
fn test_forc_test_both_logs() -> Result<(), rexpect::error::Error> {
    // Spawn the forc binary using cargo run
    let project_dir = test_fixtures_path().join("test_contract");
    let mut process = spawn(
        &format!(
            "cargo run --bin forc -- test --logs --raw-logs --path {}",
            project_dir.to_string_lossy()
        ),
        Some(TIMEOUT_MS),
    )?;

    // Assert that the output is correct
    process.exp_string("      test test_log_4")?;
    process.exp_string("decoded log values:")?;
    process.exp_string("4, log rb: 1515152261580153489")?;
    process.exp_string("raw logs:")?;
    process.exp_string(r#"[{"LogData":{"data":"0000000000000004","digest":"8005f02d43fa06e7d0585fb64c961d57e318b27a145c857bcd3a6bdb413ff7fc","id":"0000000000000000000000000000000000000000000000000000000000000000","is":10368,"len":8,"pc":11292,"ptr":12480,"ra":0,"rb":1515152261580153489}}]"#)?;
    process.exp_string("      test test_log_2")?;
    process.exp_string("decoded log values:")?;
    process.exp_string("2, log rb: 1515152261580153489")?;
    process.exp_string("raw logs:")?;
    process.exp_string(r#"[{"LogData":{"data":"0000000000000002","digest":"cd04a4754498e06db5a13c5f371f1f04ff6d2470f24aa9bd886540e5dce77f70","id":"0000000000000000000000000000000000000000000000000000000000000000","is":10368,"len":8,"pc":11292,"ptr":12480,"ra":0,"rb":1515152261580153489}}]"#)?;
    process.process.exit()?;
    Ok(())
}
