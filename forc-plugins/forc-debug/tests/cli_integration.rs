#![deny(unused_must_use)]

use escargot::CargoBuild;
use rexpect::session::spawn_command;
use std::process::Command;

#[test]
fn test_cli() {
    let port = portpicker::pick_unused_port().expect("No ports free");

    let mut fuel_core = Command::new("fuel-core")
        .arg("run")
        .arg("--debug")
        .arg("--db-type")
        .arg("in-memory")
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("Failed to start fuel-core");

    let mut run_cmd = CargoBuild::new()
        .bin("forc-debug")
        .current_release()
        .current_target()
        .run()
        .unwrap()
        .command();

    dbg!(&run_cmd);

    run_cmd.arg(format!("http://127.0.0.1:{}/graphql", port));

    let mut cmd = spawn_command(run_cmd, Some(2000)).unwrap();

    cmd.exp_regex(r"^>> ").unwrap();
    cmd.send_line("reg 0").unwrap();
    cmd.exp_regex(r"reg\[0x0\] = 0\s+# zero").unwrap();
    cmd.send_line("reg 1").unwrap();
    cmd.exp_regex(r"reg\[0x1\] = 1\s+# one").unwrap();
    cmd.send_line("breakpoint 0").unwrap();
    cmd.exp_regex(r">> ").unwrap();
    cmd.send_line("start_tx examples/example_tx.json").unwrap();
    cmd.exp_regex(r"Stopped on breakpoint at address 0 of contract 0x0{64}")
        .unwrap();
    cmd.send_line("step on").unwrap();
    cmd.exp_regex(r">> ").unwrap();
    cmd.send_line("continue").unwrap();
    cmd.exp_regex(r"Stopped on breakpoint at address 16 of contract 0x0{64}")
        .unwrap();
    cmd.send_line("step off").unwrap();
    cmd.exp_regex(r">> ").unwrap();
    cmd.send_line("continue").unwrap();
    cmd.exp_regex(r"Receipt: Return").unwrap();
    cmd.send_line("reset").unwrap();
    cmd.send_line("start_tx examples/example_tx.json").unwrap();
    cmd.exp_regex(r"Receipt: Return").unwrap();
    cmd.send_line(r"exit").unwrap();

    fuel_core.kill().expect("Couldn't kill fuel-core");
}
