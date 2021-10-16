use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span<'sc> {
    pub span: pest::Span<'sc>,
    pub(crate) path: Option<PathBuf>,
}

impl<'sc> Span<'sc> {
    pub fn start(&self) -> usize {
        self.span.start()
    }

    pub fn end(&self) -> usize {
        self.span.end()
    }

    pub fn start_pos(&self) -> pest::Position {
        self.span.start_pos()
    }

    pub fn end_pos(&self) -> pest::Position {
        self.span.end_pos()
    }

    pub fn split(&self) -> (pest::Position<'sc>, pest::Position<'sc>) {
        self.span.clone().split()
    }

    pub fn str(self) -> String {
        self.span.as_str().to_string()
    }

    pub fn as_str(&self) -> &str {
        self.span.as_str()
    }

    pub fn input(&self) -> &str {
        self.span.input()
    }

    pub fn path(&self) -> String {
        self.path
            .clone()
            .map(|p| p.into_os_string().into_string().unwrap())
            .unwrap_or_else(|| "".to_string())
    }
}
