//! Utility items shared between forc crates.

use ansiterm::Colour;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, io};
use tracing::{Level, Metadata};
pub use tracing_subscriber::{
    self,
    filter::{filter_fn, EnvFilter, LevelFilter},
    fmt::{format::FmtSpan, MakeWriter},
    layer::SubscriberExt,
};

const ACTION_COLUMN_WIDTH: usize = 12;

// Global flag to track if JSON output mode is active
static JSON_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Check if JSON mode is currently active
fn is_json_mode_active() -> bool {
    JSON_MODE_ACTIVE.load(Ordering::SeqCst)
}

/// Returns the indentation for the action prefix relative to [ACTION_COLUMN_WIDTH].
fn get_action_indentation(action: &str) -> String {
    if action.len() < ACTION_COLUMN_WIDTH {
        " ".repeat(ACTION_COLUMN_WIDTH - action.len())
    } else {
        String::new()
    }
}

enum TextStyle {
    Plain,
    Bold,
    Label(String),
    Action(String),
}

enum LogLevel {
    #[allow(dead_code)]
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Common function to handle all kinds of output with color and styling
fn print_message(text: &str, color: Colour, style: TextStyle, level: LogLevel) {
    let log_msg = match (is_json_mode_active(), style) {
        // JSON mode formatting (no colors)
        (true, TextStyle::Plain | TextStyle::Bold) => text.to_string(),
        (true, TextStyle::Label(label)) => format!("{label}: {text}"),
        (true, TextStyle::Action(action)) => {
            let indent = get_action_indentation(&action);
            format!("{indent}{action} {text}")
        }

        // Normal mode formatting (with colors)
        (false, TextStyle::Plain) => format!("{}", color.paint(text)),
        (false, TextStyle::Bold) => format!("{}", color.bold().paint(text)),
        (false, TextStyle::Label(label)) => format!("{} {}", color.bold().paint(label), text),
        (false, TextStyle::Action(action)) => {
            let indent = get_action_indentation(&action);
            format!("{}{} {}", indent, color.bold().paint(action), text)
        }
    };

    match level {
        LogLevel::Trace => tracing::trace!("{}", log_msg),
        LogLevel::Debug => tracing::debug!("{}", log_msg),
        LogLevel::Info => tracing::info!("{}", log_msg),
        LogLevel::Warn => tracing::warn!("{}", log_msg),
        LogLevel::Error => tracing::error!("{}", log_msg),
    }
}

/// Prints a label with a green-bold label prefix like "Compiling ".
pub fn println_label_green(label: &str, txt: &str) {
    print_message(
        txt,
        Colour::Green,
        TextStyle::Label(label.to_string()),
        LogLevel::Info,
    );
}

/// Prints an action message with a green-bold prefix like "   Compiling ".
pub fn println_action_green(action: &str, txt: &str) {
    println_action(action, txt, Colour::Green);
}

/// Prints a label with a red-bold label prefix like "error: ".
pub fn println_label_red(label: &str, txt: &str) {
    print_message(
        txt,
        Colour::Red,
        TextStyle::Label(label.to_string()),
        LogLevel::Info,
    );
}

/// Prints an action message with a red-bold prefix like "   Removing ".
pub fn println_action_red(action: &str, txt: &str) {
    println_action(action, txt, Colour::Red);
}

/// Prints an action message with a yellow-bold prefix like "   Finished ".
pub fn println_action_yellow(action: &str, txt: &str) {
    println_action(action, txt, Colour::Yellow);
}

fn println_action(action: &str, txt: &str, color: Colour) {
    print_message(
        txt,
        color,
        TextStyle::Action(action.to_string()),
        LogLevel::Info,
    );
}

/// Prints a warning message to stdout with the yellow prefix "warning: ".
pub fn println_warning(txt: &str) {
    print_message(
        txt,
        Colour::Yellow,
        TextStyle::Label("warning:".to_string()),
        LogLevel::Warn,
    );
}

/// Prints a warning message to stdout with the yellow prefix "warning: " only in verbose mode.
pub fn println_warning_verbose(txt: &str) {
    print_message(
        txt,
        Colour::Yellow,
        TextStyle::Label("warning:".to_string()),
        LogLevel::Debug,
    );
}

/// Prints a warning message to stderr with the red prefix "error: ".
pub fn println_error(txt: &str) {
    print_message(
        txt,
        Colour::Red,
        TextStyle::Label("error:".to_string()),
        LogLevel::Error,
    );
}

pub fn println_red(txt: &str) {
    print_message(txt, Colour::Red, TextStyle::Plain, LogLevel::Info);
}

pub fn println_green(txt: &str) {
    print_message(txt, Colour::Green, TextStyle::Plain, LogLevel::Info);
}

pub fn println_yellow(txt: &str) {
    print_message(txt, Colour::Yellow, TextStyle::Plain, LogLevel::Info);
}

pub fn println_green_bold(txt: &str) {
    print_message(txt, Colour::Green, TextStyle::Bold, LogLevel::Info);
}

pub fn println_yellow_bold(txt: &str) {
    print_message(txt, Colour::Yellow, TextStyle::Bold, LogLevel::Info);
}

pub fn println_yellow_err(txt: &str) {
    print_message(txt, Colour::Yellow, TextStyle::Plain, LogLevel::Error);
}

pub fn println_red_err(txt: &str) {
    print_message(txt, Colour::Red, TextStyle::Plain, LogLevel::Error);
}

const LOG_FILTER: &str = "RUST_LOG";

#[derive(PartialEq, Eq)]
pub enum TracingWriter {
    /// Write ERROR and WARN to stderr and everything else to stdout.
    Stdio,
    /// Write everything to stdout.
    Stdout,
    /// Write everything to stderr.
    Stderr,
    /// Write everything as structured JSON to stdout.
    Json,
}

#[derive(Default)]
pub struct TracingSubscriberOptions {
    pub verbosity: Option<u8>,
    pub silent: Option<bool>,
    pub log_level: Option<LevelFilter>,
    pub writer_mode: Option<TracingWriter>,
    pub regex_filter: Option<String>,
}

// This allows us to write ERROR and WARN level logs to stderr and everything else to stdout.
// https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/trait.MakeWriter.html
impl<'a> MakeWriter<'a> for TracingWriter {
    type Writer = Box<dyn io::Write>;

    fn make_writer(&'a self) -> Self::Writer {
        match self {
            TracingWriter::Stderr => Box::new(io::stderr()),
            // We must have an implementation of `make_writer` that makes
            // a "default" writer without any configuring metadata. Let's
            // just return stdout in that case.
            _ => Box::new(io::stdout()),
        }
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if *self == TracingWriter::Stderr
            || (*self == TracingWriter::Stdio && meta.level() <= &Level::WARN)
        {
            return Box::new(io::stderr());
        }

        // Otherwise, we'll return stdout.
        Box::new(io::stdout())
    }
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
            options.verbosity.and_then(|verbosity| match verbosity {
                1 => Some(LevelFilter::DEBUG), // matches --verbose or -v
                2 => Some(LevelFilter::TRACE), // matches -vv
                _ => None,
            })
        })
        .or_else(|| {
            options
                .silent
                .and_then(|silent| silent.then_some(LevelFilter::OFF))
        });

    let writer_mode = match options.writer_mode {
        Some(TracingWriter::Json) => {
            JSON_MODE_ACTIVE.store(true, Ordering::SeqCst);
            TracingWriter::Json
        }
        Some(TracingWriter::Stderr) => TracingWriter::Stderr,
        _ => TracingWriter::Stdio,
    };

    let builder = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_ansi(true)
        .with_level(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .with_target(false)
        .with_writer(writer_mode);

    // Use regex to filter logs - if provided; otherwise allow all logs
    let filter = filter_fn(move |metadata| {
        if let Some(ref regex_filter) = options.regex_filter {
            let regex = regex::Regex::new(regex_filter).unwrap();
            regex.is_match(metadata.target())
        } else {
            true
        }
    });

    match (is_json_mode_active(), level_filter) {
        (true, Some(level)) => {
            let subscriber = builder.json().with_max_level(level).finish().with(filter);
            tracing::subscriber::set_global_default(subscriber).expect("setting subscriber failed");
        }
        (true, None) => {
            let subscriber = builder.json().finish().with(filter);
            tracing::subscriber::set_global_default(subscriber).expect("setting subscriber failed");
        }
        (false, Some(level)) => {
            let subscriber = builder.with_max_level(level).finish().with(filter);
            tracing::subscriber::set_global_default(subscriber).expect("setting subscriber failed");
        }
        (false, None) => {
            let subscriber = builder.finish().with(filter);
            tracing::subscriber::set_global_default(subscriber).expect("setting subscriber failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    // Helper function to set up each test with consistent JSON mode state
    fn setup_test() {
        JSON_MODE_ACTIVE.store(false, Ordering::SeqCst);
    }

    #[traced_test]
    #[test]
    fn test_println_label_green() {
        setup_test();

        let txt = "main.sw";
        println_label_green("Compiling", txt);

        let expected_action = "\x1b[1;32mCompiling\x1b[0m";
        assert!(logs_contain(&format!("{expected_action} {txt}")));
    }

    #[traced_test]
    #[test]
    fn test_println_label_red() {
        setup_test();

        let txt = "main.sw";
        println_label_red("Error", txt);

        let expected_action = "\x1b[1;31mError\x1b[0m";
        assert!(logs_contain(&format!("{expected_action} {txt}")));
    }

    #[traced_test]
    #[test]
    fn test_println_action_green() {
        setup_test();

        let txt = "main.sw";
        println_action_green("Compiling", txt);

        let expected_action = "\x1b[1;32mCompiling\x1b[0m";
        assert!(logs_contain(&format!("    {expected_action} {txt}")));
    }

    #[traced_test]
    #[test]
    fn test_println_action_green_long() {
        setup_test();

        let txt = "main.sw";
        println_action_green("Supercalifragilistic", txt);

        let expected_action = "\x1b[1;32mSupercalifragilistic\x1b[0m";
        assert!(logs_contain(&format!("{expected_action} {txt}")));
    }

    #[traced_test]
    #[test]
    fn test_println_action_red() {
        setup_test();

        let txt = "main";
        println_action_red("Removing", txt);

        let expected_action = "\x1b[1;31mRemoving\x1b[0m";
        assert!(logs_contain(&format!("     {expected_action} {txt}")));
    }

    #[traced_test]
    #[test]
    fn test_json_mode_println_functions() {
        setup_test();

        JSON_MODE_ACTIVE.store(true, Ordering::SeqCst);

        // Call various print functions and capture the output
        println_label_green("Label", "Value");
        assert!(logs_contain("Label: Value"));

        println_action_green("Action", "Target");
        assert!(logs_contain("Action"));
        assert!(logs_contain("Target"));

        println_green("Green text");
        assert!(logs_contain("Green text"));

        println_warning("This is a warning");
        assert!(logs_contain("This is a warning"));

        println_error("This is an error");
        assert!(logs_contain("This is an error"));

        JSON_MODE_ACTIVE.store(false, Ordering::SeqCst);
    }
}
