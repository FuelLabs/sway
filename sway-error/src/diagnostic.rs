use std::{path::PathBuf, vec};

use sway_types::{Span, SourceId, SourceEngine};

/// Provides detailed, rich description of a compile error or warning.
#[derive(Debug, Default)]
pub struct Diagnostic {
    pub(crate) reason: Option<String>,
    pub(crate) issue: Issue,
    pub(crate) hints: Vec<Hint>,
    pub(crate) help: Vec<String>,
}

impl Diagnostic {
    pub fn level(&self) -> Level {
        match self.issue.label_type {
            LabelType::Error => Level::Error,
            LabelType::Warning => Level::Warning,
            _ => unreachable!("The diagnostic level can be only Error or Warning, and this is enforced via Diagnostics API.")
        }
    }

    pub fn reason(&self) -> Option<&String> {
        self.reason.as_ref()
    }

    pub fn issue(&self) -> &Issue {
        &self.issue
    }

    /// All the labels, potentially in different source files.
    pub fn labels(&self) -> Vec<&Label> {
        let mut labels = Vec::<&Label>::new();

        for hint in self.hints.iter().filter(|hint| hint.is_in_source()) {
            labels.push(hint);
        }

        // If the issue is in source and there are no hints that override the issue,
        // add the issue to the labels.
        if self.issue.is_in_source() && self.hints.iter().all(|hint| hint.span != self.issue.span) {
            labels.push(&self.issue);
        }

        labels
    }

    /// All the labels in the source file found at `source_path`.
    pub fn labels_in_source(&self, source_path: &SourcePath) -> Vec<&Label> {
        self.labels()
            .iter()
            // Safe unwrapping because all the labels are in source.
            .filter(|&label| label.source_path().unwrap() == source_path)
            .map(|label| *label)
            .collect()
    }

    // All the labels that occur in the same source file where the diagnostic issue occurs.
    pub fn labels_in_issue_source(&self) -> Vec<&Label> {
        if !self.issue.is_in_source() {
            return vec![];
        }

        // Safe unwrapping because the issue is in source.
        self.labels_in_source(self.issue.source_path().unwrap())
    }


    pub fn help(&self) -> &[String] {
        self.help.as_ref()
    }

    /// All the source files that are related to the diagnostic.
    /// This means the source file of the issue itself as well
    /// as source files of all the hints.
    pub fn related_sources(&self, include_issue_source: bool) -> Vec<&SourcePath> {
        let mut source_files = vec![];

        // All unwrappings are safe because we check the existence
        // either in is_in_source() or in in_source_info().
        if self.issue.is_in_source() && include_issue_source {
            source_files.push(self.issue.source_path().unwrap());
        }

        for hint in self.labels() {
            let file = hint.source_path().unwrap();

            if !include_issue_source && file == self.issue.source_path().unwrap() {
                continue;
            }

            if !source_files.contains(&file) {
                source_files.push(file)
            }
        }

        source_files
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    Warning,
    #[default]
    Error
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelType {
    #[default]
    Info,
    Warning,
    Error
}

/// Diagnostic message related to a span of source code in a source file.
/// 
/// If the message in a particular situation cannot be related to a span
/// in a known source file (e.g., when importing symbols) // TODO-IG: Check this claim.
/// the span must be set to [Span::dummy]. Such messages without a valid span
/// will be ignored.
/// 
/// E.g., a note like 'The function "{name}" is defined here.'
/// will be displayed only when we have access to the source
/// code in which the function is defined.
/// 
/// We can also have error messages that are not related to any particular
/// place in code.
#[derive(Debug)]
pub struct Label {
    label_type: LabelType,
    span: Span,
    text: String,
    friendly_text: String,
    source_path: Option<SourcePath>,
}

impl Label {
    pub fn info(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Info, span, text)
    }

    pub fn warning(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Warning, span, text)
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Error, span, text)
    }

    fn new(source_engine: &SourceEngine, label_type: LabelType, span: Span, text: String) -> Label {
        let friendly_text = Self::maybe_uwuify(text.as_str());
        let source_path = Self::get_source_path(source_engine, &span);
        Label {
            label_type,
            span,
            text,
            friendly_text,
            source_path,
        }
    }

    // TODO-IG: Remove once multi-span is implemented for warnings.
    pub fn source_id(&self) -> Option<SourceId> {
        self.span.source_id().cloned()
    }

    /// True if the `Label` is actually related to a span of source code in a source file.
    pub fn is_in_source(&self) -> bool {
        self.source_path.is_some() && (self.span.start() < self.span.end())
    }

    pub fn label_type(&self) -> LabelType {
        self.label_type
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

    pub fn source_path(&self) -> Option<&SourcePath> {
        self.source_path.as_ref()
    }

    fn get_source_path(source_engine: &SourceEngine, span: &Span) -> Option<SourcePath> {
        let path_buf = span.source_id().cloned().map(|id| source_engine.get_path(&id));
        let path_string = path_buf.as_ref().map(|p| p.to_string_lossy().to_string());

        match (path_buf, path_string) {
            (Some(path_buf), Some(path_string)) => Some(SourcePath { path_buf, path_string }),
            _ => None
        }
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

impl Default for Label {
    fn default() -> Self {
        Self {
            label_type: LabelType::Info,
            span: Span::dummy(),
            text: "".to_string(),
            friendly_text: "".to_string(),
            source_path: None,
        }
    }
}

#[derive(Debug)]
pub struct Issue {
    label: Label,
}

impl Issue {
    pub fn warning(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::warning(source_engine, span, text)
        }
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::error(source_engine, span, text)
        }
    }
}

impl Default for Issue {
    fn default() -> Self {
        Self {
            label: Label {
                label_type: LabelType::Error,
                ..Default::default()
            }
        }
    }
}

impl std::ops::Deref for Issue {
    type Target = Label;
    fn deref(&self) -> &Self::Target {
        &self.label
    }
}

#[derive(Debug, Default)]
pub struct Hint {
    label: Label,
}

impl Hint {
    pub fn info(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::info(source_engine, span, text)
        }
    }

    pub fn warning(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::warning(source_engine, span, text)
        }
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::error(source_engine, span, text)
        }
    }
}

impl std::ops::Deref for Hint {
    type Target = Label;
    fn deref(&self) -> &Self::Target {
        &self.label
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcePath {
    path_buf: PathBuf,
    path_string: String
}

impl SourcePath {
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.path_buf
    }

    pub fn as_str(&self) -> &str {
        self.path_string.as_ref()
    }
}

pub trait ToDiagnostic {
    fn to_diagnostic(&self, source_engine: &SourceEngine) -> Diagnostic;
}
