//! Utility items shared between forc crates.

use ansi_term::Colour;
use std::str;
use std::{env, io};
use tracing::{Level, Metadata};
pub use tracing_subscriber::{
    self,
    filter::{EnvFilter, LevelFilter},
    fmt::{format::FmtSpan, MakeWriter},
};

const ACTION_COLUMN_WIDTH: usize = 12;

/// Returns the indentation for the action prefix relative to [ACTION_COLUMN_WIDTH].
fn get_action_indentation(action: &str) -> String {
    if action.len() < ACTION_COLUMN_WIDTH {
        " ".repeat(ACTION_COLUMN_WIDTH - action.len())
    } else {
        "".to_string()
    }
}

/// Prints an action message with a green-bold prefix like "   Compiling ".
pub fn println_action_green(action: &str, txt: &str) {
    tracing::info!(
        "{}{} {}",
        get_action_indentation(action),
        Colour::Green.bold().paint(action),
        txt
    );
}

/// Prints an action message with a red-bold prefix like "   Removing ".
pub fn println_action_red(action: &str, txt: &str) {
    tracing::info!(
        "{}{} {}",
        get_action_indentation(action),
        Colour::Red.bold().paint(action),
        txt
    );
}

/// Prints a warning message to stdout with the yellow prefix "warning: ".
pub fn println_warning(txt: &str) {
    tracing::warn!("{}: {}", Colour::Yellow.paint("warning"), txt);
}

/// Prints a warning message to stdout with the yellow prefix "warning: " only in verbose mode.
pub fn println_warning_verbose(txt: &str) {
    tracing::debug!("{}: {}", Colour::Yellow.paint("warning"), txt);
}

/// Prints a warning message to stderr with the red prefix "error: ".
pub fn println_error(txt: &str) {
    tracing::warn!("{}: {}", Colour::Red.paint("error"), txt);
}

pub fn println_red(txt: &str) {
    println_std_out(txt, Colour::Red);
}

pub fn println_green(txt: &str) {
    println_std_out(txt, Colour::Green);
}

pub fn println_yellow_err(txt: &str) {
    println_std_err(txt, Colour::Yellow);
}

pub fn println_red_err(txt: &str) {
    println_std_err(txt, Colour::Red);
}

fn println_std_out(txt: &str, color: Colour) {
    tracing::info!("{}", color.paint(txt));
}

fn println_std_err(txt: &str, color: Colour) {
    tracing::error!("{}", color.paint(txt));
}

const LOG_FILTER: &str = "RUST_LOG";

// This allows us to write ERROR and WARN level logs to stderr and everything else to stdout.
// https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/trait.MakeWriter.html
pub struct StdioTracingWriter {
    pub writer_mode: TracingWriterMode,
}

impl<'a> MakeWriter<'a> for StdioTracingWriter {
    type Writer = Box<dyn io::Write>;

    fn make_writer(&'a self) -> Self::Writer {
        if self.writer_mode == TracingWriterMode::Stderr {
            Box::new(io::stderr())
        } else {
            // We must have an implementation of `make_writer` that makes
            // a "default" writer without any configuring metadata. Let's
            // just return stdout in that case.
            Box::new(io::stdout())
        }
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if self.writer_mode == TracingWriterMode::Stderr
            || (self.writer_mode == TracingWriterMode::Stdio && meta.level() <= &Level::WARN)
        {
            return Box::new(io::stderr());
        }

        // Otherwise, we'll return stdout.
        Box::new(io::stdout())
    }
}

#[derive(PartialEq, Eq)]
pub enum TracingWriterMode {
    /// Write ERROR and WARN to stderr and everything else to stdout.
    Stdio,
    /// Write everything to stdout.
    Stdout,
    /// Write everything to stderr.
    Stderr,
}

#[derive(Default)]
pub struct TracingSubscriberOptions {
    pub verbosity: Option<u8>,
    pub silent: Option<bool>,
    pub log_level: Option<LevelFilter>,
    pub writer_mode: Option<TracingWriterMode>,
}

/// A subscriber built from default `tracing_subscriber::fmt::SubscriberBuilder` such that it would match directly using `println!` throughout the repo.
///
/// `RUST_LOG` environment variable can be used to set different minimum level for the subscriber, default is `INFO`.
pub fn init_tracing_subscriber(options: TracingSubscriberOptions) {
    let env_filter = match env::var_os(LOG_FILTER) {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };
    let level_filter = options
        .log_level
        .or_else(|| {
            options.verbosity.and_then(|verbosity| {
                match verbosity {
                    1 => Some(LevelFilter::DEBUG), // matches --verbose or -v
                    2 => Some(LevelFilter::TRACE), // matches -vv
                    _ => None,
                }
            })
        })
        .or_else(|| {
            options
                .silent
                .and_then(|silent| if silent { Some(LevelFilter::OFF) } else { None })
        });

    let builder = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_ansi(true)
        .with_level(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .with_target(false)
        .with_writer(StdioTracingWriter {
            writer_mode: options.writer_mode.unwrap_or(TracingWriterMode::Stdio),
        });

    // If log level, verbosity, or silent mode is set, it overrides the RUST_LOG setting
    if let Some(level_filter) = level_filter {
        builder.with_max_level(level_filter).init();
    } else {
        builder.init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_println_action_green() {
        let txt = "main.sw";
        println_action_green("Compiling", txt);

        let expected_action = "\x1b[1;32mCompiling\x1b[0m";
        assert!(logs_contain(&format!("    {} {}", expected_action, txt)));
    }

    #[traced_test]
    #[test]
    fn test_println_action_green_long() {
        let txt = "main.sw";
        println_action_green("Supercalifragilistic", txt);

        let expected_action = "\x1b[1;32mSupercalifragilistic\x1b[0m";
        assert!(logs_contain(&format!("{} {}", expected_action, txt)));
    }

    #[traced_test]
    #[test]
    fn test_println_action_red() {
        let txt = "main";
        println_action_red("Removing", txt);

        let expected_action = "\x1b[1;31mRemoving\x1b[0m";
        assert!(logs_contain(&format!("     {} {}", expected_action, txt)));
    }
}
