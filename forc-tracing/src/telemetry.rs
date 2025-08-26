//! Telemetry utilities for logging to InfluxDB

pub use fuel_telemetry::*;

/// Logs an error message to telemetry
#[macro_export]
macro_rules! error_telemetry {
    ($($arg:tt)*) => {{
        #[cfg(feature = "telemetry")]
        {
            if !$crate::is_telemetry_disabled() {
                use fuel_telemetry::{telemetry, TelemetryLevel};
                telemetry(TelemetryLevel::Error, "forc", &format!($($arg)*));
            }
        }
    }};
}

/// Logs a warning message to telemetry
#[macro_export]
macro_rules! warn_telemetry {
    ($($arg:tt)*) => {{
        #[cfg(feature = "telemetry")]
        {
            if !$crate::is_telemetry_disabled() {
                use fuel_telemetry::{telemetry, TelemetryLevel};
                telemetry(TelemetryLevel::Warn, "forc", &format!($($arg)*));
            }
        }
    }};
}

/// Logs an info message to telemetry
#[macro_export]
macro_rules! info_telemetry {
    ($($arg:tt)*) => {{
        #[cfg(feature = "telemetry")]
        {
            if !$crate::is_telemetry_disabled() {
                use fuel_telemetry::{telemetry, TelemetryLevel};
                telemetry(TelemetryLevel::Info, "forc", &format!($($arg)*));
            }
        }
    }};
}

/// Logs a debug message to telemetry
#[macro_export]
macro_rules! debug_telemetry {
    ($($arg:tt)*) => {{
        #[cfg(feature = "telemetry")]
        {
            if !$crate::is_telemetry_disabled() {
                use fuel_telemetry::{telemetry, TelemetryLevel};
                telemetry(TelemetryLevel::Debug, "forc", &format!($($arg)*));
            }
        }
    }};
}
