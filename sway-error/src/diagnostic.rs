use std::{path::PathBuf, vec};

use sway_types::{SourceEngine, Span};

/// Provides detailed, rich description of a compile error or warning.
#[derive(Debug, Default)]
pub struct Diagnostic {
    pub reason: Option<Reason>, // TODO: Make mandatory once we remove all old-style warnings and errors.
    pub issue: Issue,
    pub hints: Vec<Hint>,
    pub help: Vec<String>,
}

impl Diagnostic {
    /// For backward compatibility purposes. True if the diagnostic
    /// was defined before the detailed diagnostics were introduced.
    /// An old-style diagnostic contains just the issue.
    pub fn is_old_style(&self) -> bool {
        self.reason.is_none() && self.hints.is_empty() && self.help.is_empty()
    }

    pub fn level(&self) -> Level {
        match self.issue.label_type {
            LabelType::Error => Level::Error,
            LabelType::Warning => Level::Warning,
            LabelType::Info => Level::Info,
            _ => unreachable!("The diagnostic level can be only Error, Warning, or Info, and this is enforced via Diagnostics API.")
        }
    }

    pub fn reason(&self) -> Option<&Reason> {
        self.reason.as_ref()
    }

    pub fn issue(&self) -> &Issue {
        &self.issue
    }

    /// All the labels, potentially in different source files.
    pub fn labels(&self) -> Vec<&Label> {
        let mut labels = Vec::<&Label>::new();

        if self.issue.is_in_source() {
            labels.push(&self.issue);
        }

        for hint in self.hints.iter().filter(|hint| hint.is_in_source()) {
            labels.push(hint);
        }

        labels
    }

    /// All the labels in the source file found at `source_path`.
    pub fn labels_in_source(&self, source_path: &SourcePath) -> Vec<&Label> {
        self.labels()
            .iter()
            // Safe unwrapping because all the labels are in source.
            .filter(|&label| label.source_path().unwrap() == source_path)
            .copied()
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

    pub fn help(&self) -> impl Iterator<Item = &String> + '_ {
        self.help.iter().filter(|help| !help.is_empty())
    }

    /// A help text that will never be displayed. Convenient when defining help lines
    /// that are displayed only when a condition is met.
    pub fn help_none() -> String {
        String::new()
    }

    /// Displays an empty line in the help footer.
    /// Convenient when defining visual separations within suggestions.
    pub fn help_empty_line() -> String {
        String::from(" ")
    }

    /// All the source files that are related to the diagnostic.
    /// This means the source file of the issue itself as well
    /// as source files of all the hints.
    pub fn related_sources(&self, include_issue_source: bool) -> Vec<&SourcePath> {
        let mut source_files = vec![];

        let issue_is_in_source = self.issue.is_in_source();

        // All `source_path()` unwrappings are safe because we check the existence
        // of source in case of an issue, and `self.labels()` returns
        // only labels that are in source.
        if issue_is_in_source && include_issue_source {
            source_files.push(self.issue.source_path().unwrap());
        }

        for hint in self.labels() {
            let file = hint.source_path().unwrap();

            if !include_issue_source
                && issue_is_in_source
                && file == self.issue.source_path().unwrap()
            {
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
    Info,
    Warning,
    #[default]
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelType {
    #[default]
    Info,
    Help,
    Warning,
    Error,
}

/// Diagnostic message related to a span of source code in a source file.
///
/// If the message in a particular situation cannot be related to a span
/// in a known source file (e.g., when importing symbols)
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
    source_path: Option<SourcePath>,
}

impl Label {
    pub fn info(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Info, span, text)
    }

    pub fn help(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Help, span, text)
    }

    pub fn warning(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Warning, span, text)
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> Label {
        Self::new(source_engine, LabelType::Error, span, text)
    }

    fn new(source_engine: &SourceEngine, label_type: LabelType, span: Span, text: String) -> Label {
        let source_path = Self::get_source_path(source_engine, &span);
        Label {
            label_type,
            span,
            text,
            source_path,
        }
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

    pub fn source_path(&self) -> Option<&SourcePath> {
        self.source_path.as_ref()
    }

    fn get_source_path(source_engine: &SourceEngine, span: &Span) -> Option<SourcePath> {
        let path_buf = span
            .source_id()
            .cloned()
            .map(|id| source_engine.get_path(&id));
        let path_string = path_buf.as_ref().map(|p| p.to_string_lossy().to_string());

        match (path_buf, path_string) {
            (Some(path_buf), Some(path_string)) => Some(SourcePath {
                path_buf,
                path_string,
            }),
            _ => None,
        }
    }
}

impl Default for Label {
    fn default() -> Self {
        Self {
            label_type: LabelType::Info,
            span: Span::dummy(),
            text: "".to_string(),
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
            label: Label::warning(source_engine, span, text),
        }
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::error(source_engine, span, text),
        }
    }

    pub fn info(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::info(source_engine, span, text),
        }
    }
}

impl Default for Issue {
    fn default() -> Self {
        Self {
            label: Label {
                label_type: LabelType::Error,
                ..Default::default()
            },
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
            label: Label::info(source_engine, span, text),
        }
    }

    pub fn underscored_info(source_engine: &SourceEngine, span: Span) -> Self {
        Self::info(source_engine, span, "".to_string())
    }

    pub fn multi_info(source_engine: &SourceEngine, span: &Span, hints: Vec<String>) -> Vec<Self> {
        hints
            .into_iter()
            .map(|hint| Self::info(source_engine, span.clone(), hint))
            .collect()
    }

    pub fn help(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::help(source_engine, span, text),
        }
    }

    pub fn multi_help(source_engine: &SourceEngine, span: &Span, hints: Vec<String>) -> Vec<Self> {
        hints
            .into_iter()
            .map(|hint| Self::help(source_engine, span.clone(), hint))
            .collect()
    }

    pub fn warning(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::warning(source_engine, span, text),
        }
    }

    pub fn multi_warning(
        source_engine: &SourceEngine,
        span: &Span,
        hints: Vec<String>,
    ) -> Vec<Self> {
        hints
            .into_iter()
            .map(|hint| Self::warning(source_engine, span.clone(), hint))
            .collect()
    }

    pub fn error(source_engine: &SourceEngine, span: Span, text: String) -> Self {
        Self {
            label: Label::error(source_engine, span, text),
        }
    }

    pub fn multi_error(source_engine: &SourceEngine, span: &Span, hints: Vec<String>) -> Vec<Self> {
        hints
            .into_iter()
            .map(|hint| Self::error(source_engine, span.clone(), hint))
            .collect()
    }

    /// A [Hint] that will never be displayed. Convenient when defining [Hint]s that
    /// are displayed only if a condition is met.
    pub fn none() -> Self {
        Self {
            label: Label::default(),
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
    path_string: String,
}

impl SourcePath {
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.path_buf
    }

    pub fn as_str(&self) -> &str {
        self.path_string.as_ref()
    }
}

/// Describes the different areas that we have in the
/// sway-error crate. It allows grouping of diagnostics
/// and ensuring that we have unique diagnostic code
/// numbers in each of the groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagnosticArea {
    #[default]
    LexicalAnalysis,
    Parsing,
    ParseTreeConversion,
    TypeChecking,
    SemanticAnalysis,
    Warnings,
    Migrations,
}

impl DiagnosticArea {
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::LexicalAnalysis => "E0",
            Self::Parsing => "E1",
            Self::ParseTreeConversion => "E2",
            Self::TypeChecking => "E3",
            Self::SemanticAnalysis => "E4",
            Self::Warnings => "W0",
            Self::Migrations => "M0",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Code {
    area: DiagnosticArea,
    number: u16,
    text: String,
}

impl Code {
    pub fn lexical_analysis(number: u16) -> Code {
        Self::new(DiagnosticArea::LexicalAnalysis, number)
    }

    pub fn parsing(number: u16) -> Code {
        Self::new(DiagnosticArea::Parsing, number)
    }

    pub fn parse_tree_conversion(number: u16) -> Code {
        Self::new(DiagnosticArea::ParseTreeConversion, number)
    }

    pub fn type_checking(number: u16) -> Code {
        Self::new(DiagnosticArea::TypeChecking, number)
    }

    pub fn semantic_analysis(number: u16) -> Self {
        Self::new(DiagnosticArea::SemanticAnalysis, number)
    }

    pub fn warnings(number: u16) -> Code {
        Self::new(DiagnosticArea::Warnings, number)
    }

    pub fn migrations(number: u16) -> Code {
        Self::new(DiagnosticArea::Migrations, number)
    }

    fn new(area: DiagnosticArea, number: u16) -> Self {
        debug_assert!(
            0 < number && number < 999,
            "The diagnostic code number must be greater then zero and smaller then 999."
        );
        Self {
            area,
            number,
            text: format!("{}{:03}", area.prefix(), number),
        }
    }

    pub fn as_str(&self) -> &str {
        self.text.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reason {
    code: Code,
    description: String,
}

impl Reason {
    pub fn new(code: Code, description: String) -> Self {
        Self { code, description }
    }

    pub fn code(&self) -> &str {
        self.code.as_str()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }
}

pub trait ToDiagnostic {
    fn to_diagnostic(&self, source_engine: &SourceEngine) -> Diagnostic;
}
