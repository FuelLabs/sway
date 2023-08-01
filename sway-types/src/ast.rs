#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
}

impl Delimiter {
    pub fn as_open_char(self) -> char {
        match self {
            Delimiter::Parenthesis => '(',
            Delimiter::Brace => '{',
            Delimiter::Bracket => '[',
        }
    }
    pub fn as_close_char(self) -> char {
        match self {
            Delimiter::Parenthesis => ')',
            Delimiter::Brace => '}',
            Delimiter::Bracket => ']',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PunctKind {
    Semicolon,
    Colon,
    ForwardSlash,
    Comma,
    Star,
    Add,
    Sub,
    LessThan,
    GreaterThan,
    Equals,
    Dot,
    Bang,
    Percent,
    Ampersand,
    Caret,
    Pipe,
    Underscore,
    Sharp,
}

impl PunctKind {
    pub fn as_char(&self) -> char {
        match self {
            PunctKind::Semicolon => ';',
            PunctKind::Colon => ':',
            PunctKind::ForwardSlash => '/',
            PunctKind::Comma => ',',
            PunctKind::Star => '*',
            PunctKind::Add => '+',
            PunctKind::Sub => '-',
            PunctKind::LessThan => '<',
            PunctKind::GreaterThan => '>',
            PunctKind::Equals => '=',
            PunctKind::Dot => '.',
            PunctKind::Bang => '!',
            PunctKind::Percent => '%',
            PunctKind::Ampersand => '&',
            PunctKind::Caret => '^',
            PunctKind::Pipe => '|',
            PunctKind::Underscore => '_',
            PunctKind::Sharp => '#',
        }
    }
}
