use std::process;

pub fn call_possible_forc_commands() -> Vec<String> {
    let mut possible_commands = Vec::new();
    let output = process::Command::new("forc")
        .arg("--help")
        .output()
        .expect("Failed running forc --help");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines = output_str.lines();

    let mut subcommand_is_parsed = false;

    for line in lines {
        if subcommand_is_parsed {
            let (command, _) = line.trim().split_once(' ').unwrap_or(("", ""));
            possible_commands.push(command.to_string());
        }
        if line == "SUBCOMMANDS:" {
            subcommand_is_parsed = true;
        }
    }

    possible_commands
}
