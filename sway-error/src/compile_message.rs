use sway_types::{Span, SourceId};

/// Rich description of a compile message.
/// Compile message can be a compile error or a compile warning.
///
/// `CompileMessageDescription` contains rich contextual
/// information about the message. E.g., place in code
/// where an error occurs, additional places in code
/// that are related to the error, help on how to fix the
/// error, etc.
#[derive(Debug, Default)]
pub struct CompileMessageDescription {
    pub(crate) message_type: CompileMessageType,
    pub(crate) title: Option<String>,
    pub(crate) message: SourceMessage,
}

impl CompileMessageDescription {
    pub fn message_type(&self) -> CompileMessageType {
        self.message_type
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn message(&self) -> &SourceMessage {
        &self.message
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompileMessageType {
    Warning,
    #[default]
    Error
}

/// Message that is potentially related to a span of source code in a specific file.
///
/// Hints and notes can in certain situations be
/// related to a span of source code, and in certain not.
/// E.g., a hint like 'The function "f" is defined here.'
/// makes sense only when we have access to the source
/// code in which the function `f` is defined.
/// 
/// To communicate that for a particular case the message
/// does not relate to a span, set the `span` to
/// [Span::dummy].
#[derive(Debug)]
pub struct SourceMessage {
    pub(crate) span: Span,
    pub(crate) message: String,
}

impl SourceMessage {
    pub fn source_id(&self) -> Option<SourceId> {
        self.span.source_id().cloned()
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }
}

impl Default for SourceMessage {
    fn default() -> Self {
        Self {
            span: Span::dummy(),
            message: "".to_string()
        }
    }
}

/// Represents a compile message, error or warning, that
/// can provide its detailed, rich description.
pub trait DescribableCompileMessage {
    fn description(&self) -> CompileMessageDescription;
}