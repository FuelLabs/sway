//! Telemetry utilities for logging to InfluxDB

// When telemetry feature is enabled, re-export all fuel_telemetry macros
#[cfg(feature = "telemetry")]
pub use fuel_telemetry::{
    debug_telemetry, error_telemetry, info_telemetry, span_telemetry, trace_telemetry,
    warn_telemetry,
};

// When telemetry feature is disabled, provide stub macros that trigger compile-time errors
#[cfg(not(feature = "telemetry"))]
pub use self::disabled_telemetry::*;

#[cfg(not(feature = "telemetry"))]
mod disabled_telemetry {
    /// Triggers a compile-time error when telemetry is not enabled
    #[macro_export]
    macro_rules! telemetry_disabled {
        () => {
            compile_error!(
                "Telemetry is disabled. Enable the 'telemetry' feature to use telemetry macros."
            )
        };
    }

    #[macro_export]
    macro_rules! error_telemetry {
        ($($arg:tt)*) => {
            telemetry_disabled!()
        };
    }

    #[macro_export]
    macro_rules! warn_telemetry {
        ($($arg:tt)*) => {
            telemetry_disabled!()
        };
    }

    #[macro_export]
    macro_rules! info_telemetry {
        ($($arg:tt)*) => {
            telemetry_disabled!()
        };
    }

    #[macro_export]
    macro_rules! debug_telemetry {
        ($($arg:tt)*) => {
            telemetry_disabled!()
        };
    }

    #[macro_export]
    macro_rules! trace_telemetry {
        ($($arg:tt)*) => {
            telemetry_disabled!()
        };
    }

    #[macro_export]
    macro_rules! span_telemetry {
        ($($arg:tt)*) => {
            telemetry_disabled!()
        };
    }

    pub use {
        debug_telemetry, error_telemetry, info_telemetry, span_telemetry, trace_telemetry,
        warn_telemetry,
    };
}
