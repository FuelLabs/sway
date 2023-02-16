use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub debug: DebugConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub inlay_hints: InlayHintsConfig,
    #[serde(default)]
    pub diagnostic: DiagnosticConfig,
    #[serde(default)]
    pub on_enter: OnEnterConfig,
    #[serde(default, skip_serializing)]
    trace: TraceConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
struct TraceConfig {}

// Options for debugging various parts of the server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugConfig {
    pub show_collected_tokens_as_warnings: Warnings,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            show_collected_tokens_as_warnings: Warnings::Default,
        }
    }
}

// Options for displaying compiler diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticConfig {
    pub show_warnings: bool,
    pub show_errors: bool,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            show_warnings: true,
            show_errors: true,
        }
    }
}

// Options for confguring server logging.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(with = "LevelFilterDef")]
    pub level: LevelFilter,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LevelFilter::OFF,
        }
    }
}

// This allows us to deserialize the enum that is defined in another crate.
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(remote = "LevelFilter")]
#[allow(clippy::upper_case_acronyms)]
enum LevelFilterDef {
    OFF,
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

/// Instructs the client to draw squiggly lines
/// under all of the tokens that our server managed to parse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Warnings {
    Default,
    Parsed,
    Typed,
}

// Options for configuring inlay hints.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlayHintsConfig {
    /// Whether to render leading colons for type hints, and trailing colons for parameter hints.
    pub render_colons: bool,
    /// Whether to show inlay type hints for variables.
    pub type_hints: bool,
    /// Maximum length for inlay hints. Set to null to have an unlimited length.
    pub max_length: Option<usize>,
}

impl Default for InlayHintsConfig {
    fn default() -> Self {
        Self {
            render_colons: true,
            type_hints: true,
            max_length: Some(25),
        }
    }
}

// Options for additional behavior when the user presses enter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnEnterConfig {
    pub continue_doc_comments: Option<bool>,
    pub continue_comments: Option<bool>,
}

impl Default for OnEnterConfig {
    fn default() -> Self {
        Self {
            continue_doc_comments: Some(true),
            continue_comments: Some(false),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Warnings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WarningsVisitor;

        impl<'de> serde::de::Visitor<'de> for WarningsVisitor {
            type Value = Warnings;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string representing a Warnings")
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Warnings, E> {
                Ok(match s {
                    "off" => Warnings::Default,
                    "parsed" => Warnings::Parsed,
                    "typed" => Warnings::Typed,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
                })
            }
        }

        deserializer.deserialize_any(WarningsVisitor)
    }
}
