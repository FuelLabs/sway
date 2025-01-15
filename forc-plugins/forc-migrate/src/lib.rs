pub mod cli;
#[macro_use]
mod migrations;
mod matching;
mod modifying;

use std::fmt::Display;
use std::io::Write;
use std::{io, usize, vec};

/// Returns a single error string formed of the `error` and `instructions`.
/// The returned string is formatted to be used as an error message in the [anyhow::bail] macro.
fn instructive_error<E: Display, I: Display>(error: E, instructions: &[I]) -> String {
    let mut error_message = vec![format!("{error}")];
    instructions
        .iter()
        .map(|inst| format!("       {inst}"))
        .for_each(|inst| error_message.push(inst));
    error_message.join("\n")
}

/// Returns a single error string representing an internal error.
/// The returned string is formatted to be used as an error message in the [anyhow::bail] macro.
fn internal_error<E: Display>(error: E) -> String {
    instructive_error(error, &[
        "This is an internal error and signifies a bug in the `forc migrate` tool.",
        "Please report this error by filing an issue at https://github.com/FuelLabs/sway/issues/new?template=bug_report.yml.",
    ])
}

/// Prints a menu containing numbered `options` and asks to choose one of them.
/// Returns zero-indexed index of the chosen option.
fn print_single_choice_menu<S: AsRef<str> + Display>(options: &[S]) -> usize {
    assert!(options.len() > 1, "There must be at least two options to choose from.");

    for (i, option) in options.iter().enumerate() {
        println!("{}. {option}", i+1);
    }

    let mut choice = usize::MAX;
    while choice == 0 || choice > options.len() {
        print!("Enter your choice [1..{}]: ", options.len());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        choice = match std::io::stdin().read_line(&mut input) {
            Ok(_) => match input.trim().parse() {
                Ok(choice) => choice,
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }

    choice - 1
}
