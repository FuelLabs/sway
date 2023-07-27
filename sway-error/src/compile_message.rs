use std::{path::PathBuf, vec};

use sway_types::{Span, SourceId, SourceEngine};

/// Provides detailed, rich description of a compile error or warning.
///
/// `CompileMessage` contains detailed contextual
/// information about a compile message. E.g., place in code
/// where an error occurs, additional places in code
/// that are related to the error, help on how to fix the
/// error, etc.
#[derive(Debug, Default)]
pub struct CompileMessage {
    pub(crate) message_type: CompileMessageType,
    pub(crate) title: Option<String>,
    pub(crate) message: InSourceMessage,
    pub(crate) in_source_info: Vec<InSourceMessage>,
    pub(crate) help: Vec<String>,
}

impl CompileMessage {
    pub fn message_type(&self) -> CompileMessageType {
        self.message_type
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn message(&self) -> &InSourceMessage {
        &self.message
    }

    /// All info messages that potentially appear within the lines of source
    /// code. By convention, if the [InSourceMessage::is_in_source]
    /// of a particular message returns false, the message
    /// should not be displayed. To get the in source info messages that
    /// should be displayed use [InSourceMessage::in_source_info].
    pub fn all_in_source_info(&self) -> &[InSourceMessage] {
        self.in_source_info.as_ref()
    }

    /// All info messages that appear within the lines of source code.
    pub fn in_source_info(&self) -> impl Iterator<Item = &InSourceMessage> + '_ {
        self.in_source_info.iter().filter(|message| message.is_in_source())
    }

    pub fn help(&self) -> &[String] {
        self.help.as_ref()
    }

    /// All the source files that are related to the message.
    /// The error or warning itself, together with the notes
    /// are all contained in these source files.
    pub fn related_source_files(&self, include_message_file: bool) -> Vec<(&PathBuf, &String)> {
        let mut source_files = vec![];

        // All unwrappings are safe because we check the existince
        // either in is_in_source() or in in_source_info().
        if self.message.is_in_source() && include_message_file {
            source_files.push((
                self.message.source_file_path().unwrap(),
                self.message.source_file_path_as_string().unwrap()
            ));
        }

        for note in self.in_source_info() {
            let file = (
                note.source_file_path().unwrap(),
                note.source_file_path_as_string().unwrap()
            );

            if !include_message_file && file.1 == self.message.source_file_path_as_string().unwrap() {
                continue;
            }

            if !source_files.contains(&file) {
                source_files.push(file)
            }
        }

        source_files
    }

    pub fn source_messages_in_file(&self, file_path: &String) -> Vec<&InSourceMessage> {
        let mut messages = vec![];

        if self.message.is_in_source() && self.message.source_file_path_as_string().unwrap() == file_path {
            messages.push(&self.message);
        }

        for note in self.in_source_info() {
            if note.source_file_path_as_string().unwrap() == file_path {
                messages.push(note);
            }
        }

        messages
    }


    pub fn source_messages_in_message_file(&self) -> Vec<&InSourceMessage> {
        if !self.message.is_in_source() {
            return vec![];
        }

        self.source_messages_in_file(self.message.source_file_path_as_string().unwrap())
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompileMessageType {
    Warning,
    #[default]
    Error
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InSourceMessageType {
    #[default]
    Info,
    Warning,
    Error
}

/// Message that is potentially related to a span of source code in a specific file.
///
/// What does it mean _potentially_ related?
/// 
/// Hints and notes can in certain situations be
/// related to a span of source code, and in certain not.
/// E.g., a hint like 'The function "f" is defined here.'
/// makes sense only when we have access to the source
/// code in which the function `f` is defined.
/// 
/// We can have error messages that are not realted to any particular
/// place in code.
/// 
/// To communicate that for a particular case the message
/// does not relate to a span, set the `span` to [Span::dummy].
#[derive(Debug)]
pub struct InSourceMessage {
    // TODO-IG: Use Option to clearly communicate intention? Cumbersome to use now when messages
    //          are constructed imperatively. And we need to support Span::dummy() so far for
    //          backward compatibility.
    message_type: InSourceMessageType,
    span: Span,
    text: String,
    friendly_text: String,
    source_file_path: Option<PathBuf>,
    source_file_path_as_string: Option<String>,
}

impl InSourceMessage {
    pub fn info(source_engine: &SourceEngine, span: Span, text: String) -> InSourceMessage {
        Self::new(source_engine, InSourceMessageType::Info, span, text)
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> InSourceMessage {
        Self::new(source_engine, InSourceMessageType::Error, span, text)
    }

    fn new(source_engine: &SourceEngine, message_type: InSourceMessageType, span: Span, text: String) -> InSourceMessage {
        let (path, path_str) = Self::get_source_file_paths(source_engine, &span);
        let friendly_text = Self::maybe_uwuify(text.as_str());
        InSourceMessage {
            message_type,
            span,
            text,
            friendly_text,
            source_file_path: path,
            source_file_path_as_string: path_str
        }
    }

    // TODO-IG: Remove once multi-span is implemented for warnings.
    pub fn source_id(&self) -> Option<SourceId> {
        self.span.source_id().cloned()
    }

    /// True if the `InSourceMessage` is actually related to a span of source code in a specific file.
    pub fn is_in_source(&self) -> bool {
        self.source_file_path.is_some() && (self.span.start() < self.span.end())
    }

    pub fn message_type(&self) -> InSourceMessageType {
        self.message_type
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub fn friendly_text(&self) -> &str {
        self.friendly_text.as_ref()
    }

    pub fn source_file_path(&self) -> Option<&PathBuf> {
        self.source_file_path.as_ref()
    }

    pub fn source_file_path_as_string(&self) -> Option<&String> {
        self.source_file_path_as_string.as_ref()
    }

    fn get_source_file_paths(source_engine: &SourceEngine, span: &Span) -> (Option<PathBuf>, Option<String>) {
        let path_buf = span.source_id().cloned().map(|id| source_engine.get_path(&id));
        let path_str = path_buf.as_ref().map(|p| p.to_string_lossy().to_string());
        (path_buf, path_str)
    }

    #[cfg(all(feature = "uwu", any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn maybe_uwuify(raw: &str) -> String {
        use uwuifier::uwuify_str_sse;
        uwuify_str_sse(raw)
    }

    #[cfg(all(feature = "uwu", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    pub fn maybe_uwuify(raw: &str) -> String {
        compile_error!("The `uwu` feature only works on x86 or x86_64 processors.");
        Default::default()
    }

    #[cfg(not(feature = "uwu"))]
    pub fn maybe_uwuify(raw: &str) -> String {
        raw.to_string()
    }
}

impl Default for InSourceMessage {
    fn default() -> Self {
        Self {
            message_type: InSourceMessageType::Info,
            span: Span::dummy(),
            text: "".to_string(),
            friendly_text: "".to_string(),
            source_file_path: None,
            source_file_path_as_string: None,
        }
    }
}

pub trait ToCompileMessage {
    fn to_compile_message(&self, source_engine: &SourceEngine) -> CompileMessage;
}
