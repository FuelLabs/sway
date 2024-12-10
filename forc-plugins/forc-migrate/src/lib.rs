pub mod cli;
#[macro_use]
mod migrations;

use std::fmt::Display;
use std::vec;

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
