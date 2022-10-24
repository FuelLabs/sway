use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub debug:       Debug,
    pub inlay_hints: InlayHintsConfig,
    #[serde(skip_serializing)]
    trace:           Trace,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
struct Trace {}

// Options for debugging various parts of the server
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Debug {
    pub show_collected_tokens_as_warnings: Warnings,
}

/// Instructs the client to draw squiggly lines
/// under all of the tokens that our server managed to parse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Warnings {
    Default,
    Parsed,
    Typed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlayHintsConfig {
    /// Whether to render leading colons for type hints, and trailing colons for parameter hints.
    pub render_colons: bool,
    /// Whether to show inlay type hints for variables.
    pub type_hints:    bool,
    /// Maximum length for inlay hints. Set to null to have an unlimited length.
    pub max_length:    Option<usize>,
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            show_collected_tokens_as_warnings: Warnings::Default,
        }
    }
}

impl Default for InlayHintsConfig {
    fn default() -> Self {
        Self {
            render_colons: true,
            type_hints:    true,
            max_length:    Some(25),
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
