//! Markdown formatting.
//!
//! Sometimes, we want to display a "rich text" in the UI. At the moment, we use
//! markdown for this purpose.
//! Modified from rust-analyzer.
use crate::{
    capabilities::hover::hover_link_contents::RelatedType, config::LspClient,
    core::token::get_range_from_span, utils::document::get_url_from_span,
};
use serde_json::{json, Value};
use std::fmt::{self};
use sway_types::{SourceEngine, Span};
use urlencoding::encode;

const GO_TO_COMMAND: &str = "sway.goToLocation";
const PEEK_COMMAND: &str = "sway.peekLocations";

/// A handy wrapper around `String` for constructing markdown documents.
#[derive(Default, Debug)]
pub struct Markup {
    text: String,
}

impl From<Markup> for String {
    fn from(markup: Markup) -> Self {
        markup.text
    }
}

impl From<String> for Markup {
    fn from(text: String) -> Self {
        Markup { text }
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl Markup {
    /// Creates a new empty `Markup`.
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    /// If contents is `Some`, format the contents within a sway code block.
    pub fn maybe_add_sway_block(self, contents: Option<String>) -> Self {
        match contents {
            Some(contents) => self.fenced_sway_block(&contents),
            None => self,
        }
    }

    /// Adds go-to links if there are any related types, a link to view implementations if there are any,
    /// or nothing if there are no related types or implementations. Only adds links for VSCode clients.
    pub fn maybe_add_links(
        self,
        source_engine: &SourceEngine,
        related_types: &[RelatedType],
        implementations: &[Span],
        client_config: &LspClient,
    ) -> Self {
        if client_config != &LspClient::VsCode {
            return self;
        }

        if related_types.is_empty() {
            let locations = implementations
                .iter()
                .filter_map(|span| {
                    if let Ok(uri) = get_url_from_span(source_engine, span) {
                        let range = get_range_from_span(span);
                        Some(json!({ "uri": uri, "range": range }))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if locations.len() > 1 {
                let args = json!([{ "locations": locations }]);
                let links_string = format!(
                    "[{} implementations]({} {})",
                    locations.len(),
                    command_uri(PEEK_COMMAND, &args),
                    quoted_tooltip("Go to implementations")
                );
                self.text(&links_string)
            } else {
                self
            }
        } else {
            let links_string = related_types
                .iter()
                .map(|related_type| {
                    let args = json!([{ "uri": related_type.uri, "range": &related_type.range }]);
                    format!(
                        "[{}]({} {})",
                        related_type.name,
                        command_uri(GO_TO_COMMAND, &args),
                        quoted_tooltip(&related_type.callpath.to_string())
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            self.text(&format!("Go to {links_string}"))
        }
    }

    /// Contents will be formatted with sway syntax highlighting.
    pub fn fenced_sway_block(self, contents: &impl fmt::Display) -> Self {
        let code_block = format!("```sway\n{contents}\n```");
        self.text(&code_block)
    }

    /// Add text to the markup.
    pub fn text(self, contents: &str) -> Self {
        if !self.text.is_empty() {
            return self.line_separator().push_str(contents);
        }
        self.push_str(contents)
    }

    /// Add text without a line separator.
    fn push_str(mut self, contents: &str) -> Self {
        self.text.push_str(contents);
        self
    }

    /// Add a new section.
    fn line_separator(mut self) -> Self {
        self.text.push_str("\n---\n");
        self
    }

    /// Get the inner string as a str.
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}

/// Builds a markdown URI using the "command" scheme and args passed as encoded JSON.
fn command_uri(command: &str, args: &Value) -> String {
    format!("command:{}?{}", command, encode(args.to_string().as_str()))
}

fn quoted_tooltip(text: &str) -> String {
    format!("\"{text}\"")
}
