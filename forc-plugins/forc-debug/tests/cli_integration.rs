#![deny(unused_must_use)]
use escargot::CargoBuild;
use rexpect::session::spawn_command;
use std::process::Command;

#[test]
fn test_cli() {
    let port = portpicker::pick_unused_port().expect("No ports free");
    #[allow(clippy::zombie_processes)]
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

    run_cmd.arg(format!("http://127.0.0.1:{port}/graphql"));

    // Increased timeout to account for rustyline initialization
    let mut cmd = spawn_command(run_cmd, Some(5000)).unwrap();

    // Handle rustyline's escape sequences before the prompt
    cmd.exp_string("\u{1b}[?2004h").unwrap();

    // Green >> prompt
    let prompt = "\u{1b}[38;2;4;234;130m>>\u{1b}[0m ";

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("reg 0").unwrap();
    cmd.exp_regex(r"reg\[0x0\] = 0\s+# zero").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("reg 1").unwrap();
    cmd.exp_regex(r"reg\[0x1\] = 1\s+# one").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("breakpoint 0").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("start_tx examples/example_tx.json examples/example_abi.json")
        .unwrap();
    cmd.exp_regex(r"Stopped on breakpoint at address 0 of contract 0x0{64}")
        .unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("step on").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("continue").unwrap();
    cmd.exp_regex(r"Stopped on breakpoint at address 4 of contract 0x0{64}")
        .unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("step off").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("continue").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("reset").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("start_tx examples/example_tx.json examples/example_abi.json")
        .unwrap();
    cmd.exp_regex(r"Decoded log value: 120").unwrap();

    cmd.exp_string(prompt).unwrap();
    cmd.send_line("quit").unwrap();
    fuel_core.kill().expect("Couldn't kill fuel-core");
}
