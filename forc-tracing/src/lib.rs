//! Utility items shared between forc crates.

use ansiterm::Colour;
use std::str;
use std::{env, io};
use tracing::{Level, Metadata, Subscriber};
pub use tracing_subscriber::{
    self,
    filter::{EnvFilter, LevelFilter},
    fmt::{format::FmtSpan, MakeWriter},
    layer::{Context, Filter},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[cfg(feature = "telemetry")]
use fuel_telemetry::WorkerGuard;

#[cfg(feature = "telemetry")]
pub mod telemetry {
    pub use fuel_telemetry::{
        debug_telemetry, error_telemetry, info_telemetry, span_telemetry, trace_telemetry,
        warn_telemetry,
    };
}

#[macro_export]
macro_rules! telemetry_disabled {
    () => {
        compile_error!("Telemetry is disabled. Add `features = [\"telemetry\"]` to the `forc-tracing` dependency to enable telemetry");
    }
}

#[cfg(not(feature = "telemetry"))]
pub mod telemetry {
    #[macro_export]
    macro_rules! error_telemetry {
        ($($arg:tt)*) => {
            $crate::telemetry_disabled!();
        };
    }

    #[macro_export]
    macro_rules! info_telemetry {
        ($($arg:tt)*) => {
            $crate::telemetry_disabled!();
        };
    }

    #[macro_export]
    macro_rules! warn_telemetry {
        ($($arg:tt)*) => {
            $crate::telemetry_disabled!();
        };
    }

    #[macro_export]
    macro_rules! debug_telemetry {
        ($($arg:tt)*) => {
            $crate::telemetry_disabled!();
        };
    }

    #[macro_export]
    macro_rules! trace_telemetry {
        ($($arg:tt)*) => {
            $crate::telemetry_disabled!();
        };
    }

    #[macro_export]
    macro_rules! span_telemetry {
        ($($arg:tt)*) => {
            $crate::telemetry_disabled!();
        };
    }

    pub use {
        debug_telemetry, error_telemetry, info_telemetry, span_telemetry, trace_telemetry,
        warn_telemetry,
    };
}

const ACTION_COLUMN_WIDTH: usize = 12;

/// Returns the indentation for the action prefix relative to [ACTION_COLUMN_WIDTH].
fn get_action_indentation(action: &str) -> String {
    if action.len() < ACTION_COLUMN_WIDTH {
        " ".repeat(ACTION_COLUMN_WIDTH - action.len())
    } else {
        "".to_string()
    }
}

/// Prints a label with a green-bold label prefix like "Compiling ".
pub fn println_label_green(label: &str, txt: &str) {
    println_label(label, txt, Colour::Green);
}

/// Prints an action message with a green-bold prefix like "   Compiling ".
pub fn println_action_green(action: &str, txt: &str) {
    println_action(action, txt, Colour::Green);
}

/// Prints a label with a red-bold label prefix like "error: ".
pub fn println_label_red(label: &str, txt: &str) {
    println_action(label, txt, Colour::Red);
}

fn println_label(label: &str, txt: &str, color: Colour) {
    tracing::info!("{} {}", color.bold().paint(label), txt);
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
    tracing::info!(
        "{}{} {}",
        get_action_indentation(action),
        color.bold().paint(action),
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

pub fn println_yellow(txt: &str) {
    println_std_out(txt, Colour::Yellow);
}

pub fn println_green_bold(txt: &str) {
    tracing::info!("{}", Colour::Green.bold().paint(txt));
}

pub fn println_yellow_bold(txt: &str) {
    tracing::info!("{}", Colour::Yellow.bold().paint(txt));
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

struct HideTelemetryFilter<S: Subscriber> {
    _marker: std::marker::PhantomData<S>,
}

impl<S: Subscriber> HideTelemetryFilter<S> {
    fn new<F: Filter<S>>(_inner: F) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S> Filter<S> for HideTelemetryFilter<S>
where
    S: Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn enabled(&self, _meta: &Metadata<'_>, ctx: &Context<'_, S>) -> bool {
        if let Some(span) = ctx.lookup_current() {
            if span
                .fields()
                .iter()
                .any(|field| field.name() == "telemetry")
            {
                return false;
            }
        }

        true
    }
}

/// A subscriber built from default `tracing_subscriber::fmt::SubscriberBuilder` such that it would match directly using `println!` throughout the repo.
///
/// `RUST_LOG` environment variable can be used to set different minimum level for the subscriber, default is `INFO`.
pub fn init_tracing_subscriber(options: TracingSubscriberOptions) {
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

    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_level(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .with_target(false)
        .with_writer(StdioTracingWriter {
            writer_mode: options.writer_mode.unwrap_or(TracingWriterMode::Stdio),
        });

    #[cfg(feature = "telemetry")]
    let (telemetry_layer, telemetry_guard) = fuel_telemetry::new_with_watchers!().unwrap();

    // If log level, verbosity, or silent mode is set, it overrides the RUST_LOG setting
    if let Some(level_filter) = level_filter {
        let hide_telemetry_filter = HideTelemetryFilter::new(level_filter);

        #[cfg(feature = "telemetry")]
        tracing_subscriber::registry()
            .with(telemetry_layer)
            .with(layer.with_filter(hide_telemetry_filter))
            .init();

        #[cfg(not(feature = "telemetry"))]
        tracing_subscriber::registry()
            .with(layer.with_filter(hide_telemetry_filter))
            .init();
    } else {
        let env_filter = match env::var_os(LOG_FILTER) {
            Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
            None => EnvFilter::new("info"),
        };

        let hide_telemetry_filter = HideTelemetryFilter::new(env_filter);

        #[cfg(feature = "telemetry")]
        tracing_subscriber::registry()
            .with(telemetry_layer)
            .with(layer.with_filter(hide_telemetry_filter))
            .init();

        #[cfg(not(feature = "telemetry"))]
        tracing_subscriber::registry()
            .with(layer.with_filter(hide_telemetry_filter))
            .init();
    }

    #[cfg(feature = "telemetry")]
    {
        // When the process ends, Thread Local Storage will drop this guard allowing
        // the tracing appender to flush any remaining telemetry to disk.
        thread_local! {
            static GUARD: std::cell::RefCell<Option<WorkerGuard>> = const { std::cell::RefCell::new(None) };
        }

        GUARD.set(Some(telemetry_guard));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_println_label_green() {
        let txt = "main.sw";
        println_label_green("Compiling", txt);

        let expected_action = "\x1b[1;32mCompiling\x1b[0m";
        assert!(logs_contain(&format!("{} {}", expected_action, txt)));
    }

    #[traced_test]
    #[test]
    fn test_println_label_red() {
        let txt = "main.sw";
        println_label_red("Error", txt);

        let expected_action = "\x1b[1;31mError\x1b[0m";
        assert!(logs_contain(&format!("{} {}", expected_action, txt)));
    }

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
