use crate::priv_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Spacing {
    Joint,
    Alone,
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
    Tilde,
    Underscore,
    Sharp,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Punct {
    pub span: Span,
    pub kind: PunctKind,
    pub spacing: Spacing,
}

impl Spanned for Punct {
    fn span(&self) -> Span {
        self.span.clone()
    }
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
            PunctKind::Tilde => '~',
            PunctKind::Underscore => '_',
            PunctKind::Sharp => '#',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct GenericGroup<T> {
    pub delimiter: Delimiter,
    pub token_stream: T,
    pub span: Span,
}

pub type Group = GenericGroup<TokenStream>;
pub type CommentedGroup = GenericGroup<CommentedTokenStream>;

impl<T> Spanned for GenericGroup<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Comment {
    pub span: Span,
}

impl Spanned for Comment {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

/// Allows for generalizing over commented and uncommented token streams.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum GenericTokenTree<T> {
    Punct(Punct),
    Ident(Ident),
    Group(GenericGroup<T>),
    Literal(Literal),
}

pub type TokenTree = GenericTokenTree<TokenStream>;
pub type CommentedTree = GenericTokenTree<CommentedTokenStream>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum CommentedTokenTree {
    Comment(Comment),
    Tree(CommentedTree),
}

impl CommentedGroup {
    pub fn strip_comments(self) -> Group {
        Group {
            delimiter: self.delimiter,
            token_stream: self.token_stream.strip_comments(),
            span: self.span,
        }
    }
}

impl<T> Spanned for GenericTokenTree<T> {
    fn span(&self) -> Span {
        match self {
            Self::Punct(punct) => punct.span(),
            Self::Ident(ident) => ident.span(),
            Self::Group(group) => group.span(),
            Self::Literal(literal) => literal.span(),
        }
    }
}

impl Spanned for CommentedTokenTree {
    fn span(&self) -> Span {
        match self {
            Self::Comment(cmt) => cmt.span(),
            Self::Tree(tt) => tt.span(),
        }
    }
}

impl<T> From<Punct> for GenericTokenTree<T> {
    fn from(punct: Punct) -> Self {
        Self::Punct(punct)
    }
}

impl<T> From<Ident> for GenericTokenTree<T> {
    fn from(ident: Ident) -> Self {
        Self::Ident(ident)
    }
}

impl<T> From<GenericGroup<T>> for GenericTokenTree<T> {
    fn from(group: GenericGroup<T>) -> Self {
        Self::Group(group)
    }
}

impl<T> From<Literal> for GenericTokenTree<T> {
    fn from(lit: Literal) -> Self {
        Self::Literal(lit)
    }
}

impl From<Comment> for CommentedTokenTree {
    fn from(comment: Comment) -> Self {
        Self::Comment(comment)
    }
}

impl From<CommentedTree> for CommentedTokenTree {
    fn from(tree: CommentedTree) -> Self {
        Self::Tree(tree)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct TokenStream {
    token_trees: Vec<TokenTree>,
    full_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct CommentedTokenStream {
    token_trees: Vec<CommentedTokenTree>,
    full_span: Span,
}

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
#[error("{}", kind)]
pub struct LexError {
    span: Span,
    kind: LexErrorKind,
}

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum LexErrorKind {
    #[error("unclosed multiline comment")]
    UnclosedMultilineComment { unclosed_indices: Vec<usize> },
    #[error("unexpected close delimiter")]
    UnexpectedCloseDelimiter {
        position: usize,
        close_delimiter: Delimiter,
    },
    #[error("mismatched delimiters")]
    MismatchedDelimiters {
        open_position: usize,
        close_position: usize,
        open_delimiter: Delimiter,
        close_delimiter: Delimiter,
    },
    #[error("unclosed delimiter")]
    UnclosedDelimiter {
        open_position: usize,
        open_delimiter: Delimiter,
    },
    #[error("unclosed string literal")]
    UnclosedStringLiteral { position: usize },
    #[error("unclosed char literal")]
    UnclosedCharLiteral { position: usize },
    #[error("expected close quote")]
    ExpectedCloseQuote { position: usize },
    #[error("incomplete hex int literal")]
    IncompleteHexIntLiteral { position: usize },
    #[error("incomplete binary int literal")]
    IncompleteBinaryIntLiteral { position: usize },
    #[error("incomplete octal int literal")]
    IncompleteOctalIntLiteral { position: usize },
    #[error("invalid int suffix: {}", suffix)]
    InvalidIntSuffix { suffix: Ident },
    #[error("invalid character")]
    InvalidCharacter { position: usize, character: char },
    #[error("invalid hex escape")]
    InvalidHexEscape,
    #[error("unicode escape missing brace")]
    UnicodeEscapeMissingBrace { position: usize },
    #[error("invalid unicode escape digit")]
    InvalidUnicodeEscapeDigit { position: usize },
    #[error("unicode escape out of range")]
    UnicodeEscapeOutOfRange { position: usize },
    #[error("unicode escape represents an invalid char value")]
    UnicodeEscapeInvalidCharValue { span: Span },
    #[error("invalid escape code")]
    InvalidEscapeCode { position: usize },
}

impl Spanned for LexError {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl LexError {
    pub fn span_ref(&self) -> &Span {
        &self.span
    }
}

#[extension_trait]
impl CharExt for char {
    fn as_open_delimiter(self) -> Option<Delimiter> {
        match self {
            '(' => Some(Delimiter::Parenthesis),
            '{' => Some(Delimiter::Brace),
            '[' => Some(Delimiter::Bracket),
            _ => None,
        }
    }

    fn as_close_delimiter(self) -> Option<Delimiter> {
        match self {
            ')' => Some(Delimiter::Parenthesis),
            '}' => Some(Delimiter::Brace),
            ']' => Some(Delimiter::Bracket),
            _ => None,
        }
    }

    fn as_punct_kind(self) -> Option<PunctKind> {
        match self {
            ';' => Some(PunctKind::Semicolon),
            ':' => Some(PunctKind::Colon),
            '/' => Some(PunctKind::ForwardSlash),
            ',' => Some(PunctKind::Comma),
            '*' => Some(PunctKind::Star),
            '+' => Some(PunctKind::Add),
            '-' => Some(PunctKind::Sub),
            '<' => Some(PunctKind::LessThan),
            '>' => Some(PunctKind::GreaterThan),
            '=' => Some(PunctKind::Equals),
            '.' => Some(PunctKind::Dot),
            '!' => Some(PunctKind::Bang),
            '%' => Some(PunctKind::Percent),
            '&' => Some(PunctKind::Ampersand),
            '^' => Some(PunctKind::Caret),
            '|' => Some(PunctKind::Pipe),
            '~' => Some(PunctKind::Tilde),
            '_' => Some(PunctKind::Underscore),
            '#' => Some(PunctKind::Sharp),
            _ => None,
        }
    }
}

struct CharIndicesInner<'a> {
    src: &'a str,
    position: usize,
}

impl<'a> Iterator for CharIndicesInner<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<(usize, char)> {
        let mut char_indices = self.src[self.position..].char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => return None,
        };
        let ret = (self.position, c);
        match char_indices.next() {
            Some((char_width, _)) => self.position += char_width,
            None => self.position = self.src.len(),
        };
        Some(ret)
    }
}

type CharIndices<'a> = std::iter::Peekable<CharIndicesInner<'a>>;

pub fn lex(
    src: &Arc<str>,
    start: usize,
    end: usize,
    path: Option<Arc<PathBuf>>,
) -> Result<TokenStream, LexError> {
    lex_commented(src, start, end, path).map(|stream| stream.strip_comments())
}

pub fn lex_commented(
    src: &Arc<str>,
    start: usize,
    end: usize,
    path: Option<Arc<PathBuf>>,
) -> Result<CommentedTokenStream, LexError> {
    let mut char_indices = CharIndicesInner {
        src: &src[..end],
        position: start,
    }
    .peekable();
    let mut parent_token_trees = Vec::new();
    let mut token_trees: Vec<CommentedTokenTree> = Vec::new();
    while let Some((mut index, mut character)) = char_indices.next() {
        if character.is_whitespace() {
            continue;
        }
        if character == '/' {
            match char_indices.peek() {
                Some((_, '/')) => {
                    let _ = char_indices.next();
                    for (end, character) in char_indices.by_ref() {
                        if character == '\n' {
                            let span = Span::new(src.clone(), index, end, path.clone()).unwrap();
                            let comment = Comment { span };
                            token_trees.push(comment.into());
                            break;
                        }
                    }
                }
                Some((_, '*')) => {
                    let _ = char_indices.next();
                    let mut unclosed_indices = vec![index];

                    let unclosed_multiline_comment = |unclosed_indices: Vec<_>| {
                        let span = Span::new(
                            src.clone(),
                            *unclosed_indices.last().unwrap(),
                            src.len() - 1,
                            path.clone(),
                        )
                        .unwrap();
                        LexError {
                            kind: LexErrorKind::UnclosedMultilineComment { unclosed_indices },
                            span,
                        }
                    };

                    loop {
                        match char_indices.next() {
                            None => return Err(unclosed_multiline_comment(unclosed_indices)),
                            Some((_, '*')) => match char_indices.next() {
                                None => return Err(unclosed_multiline_comment(unclosed_indices)),
                                Some((slash_ix, '/')) => {
                                    let start = unclosed_indices.pop().unwrap();
                                    let end = slash_ix + '/'.len_utf8();
                                    let span =
                                        Span::new(src.clone(), start, end, path.clone()).unwrap();
                                    let comment = Comment { span };
                                    token_trees.push(comment.into());
                                    if unclosed_indices.is_empty() {
                                        break;
                                    }
                                }
                                Some((_, _)) => (),
                            },
                            Some((next_index, '/')) => match char_indices.next() {
                                None => return Err(unclosed_multiline_comment(unclosed_indices)),
                                Some((_, '*')) => {
                                    unclosed_indices.push(next_index);
                                }
                                Some((_, _)) => (),
                            },
                            Some((_, _)) => (),
                        }
                    }
                }
                Some(&(end, next_character)) => {
                    let spacing = if let Some(..) = next_character.as_punct_kind() {
                        Spacing::Joint
                    } else {
                        Spacing::Alone
                    };
                    let span = Span::new(src.clone(), start, end, path.clone()).unwrap();
                    let punct = Punct {
                        kind: PunctKind::ForwardSlash,
                        spacing,
                        span,
                    };
                    token_trees.push(CommentedTokenTree::Tree(punct.into()));
                }
                None => {
                    let span = Span::new(src.clone(), start, end, path.clone()).unwrap();
                    let punct = Punct {
                        kind: PunctKind::ForwardSlash,
                        spacing: Spacing::Alone,
                        span,
                    };
                    token_trees.push(CommentedTokenTree::Tree(punct.into()));
                }
            }
            continue;
        }
        if character.is_xid_start() || character == '_' {
            let is_raw_ident = character == 'r' && matches!(char_indices.peek(), Some((_, '#')));
            if is_raw_ident {
                char_indices.next();
                if let Some((_, next_character)) = char_indices.peek() {
                    character = *next_character;
                    if let Some((next_index, _)) = char_indices.next() {
                        index = next_index;
                    }
                }
            }
            let is_single_underscore = character == '_'
                && match char_indices.peek() {
                    Some((_, next_character)) => !next_character.is_xid_continue(),
                    None => true,
                };
            if !is_single_underscore {
                while let Some((_, next_character)) = char_indices.peek() {
                    if !next_character.is_xid_continue() {
                        break;
                    }
                    let _ = char_indices.next();
                }
                let span = span_until(src, index, &mut char_indices, &path);
                let ident = Ident::new_with_raw(span, is_raw_ident);
                token_trees.push(CommentedTokenTree::Tree(ident.into()));
                continue;
            }
        }
        if let Some(delimiter) = character.as_open_delimiter() {
            let token_trees = mem::take(&mut token_trees);
            parent_token_trees.push((token_trees, index, delimiter));
            continue;
        }
        if let Some(close_delimiter) = character.as_close_delimiter() {
            match parent_token_trees.pop() {
                None => {
                    return Err(LexError {
                        kind: LexErrorKind::UnexpectedCloseDelimiter {
                            position: index,
                            close_delimiter,
                        },
                        span: Span::new(
                            src.clone(),
                            index,
                            index + character.len_utf8(),
                            path.clone(),
                        )
                        .unwrap(),
                    });
                }
                Some((mut parent, open_index, open_delimiter)) => {
                    if open_delimiter != close_delimiter {
                        return Err(LexError {
                            kind: LexErrorKind::MismatchedDelimiters {
                                open_position: open_index,
                                close_position: index,
                                open_delimiter,
                                close_delimiter,
                            },
                            span: Span::new(
                                src.clone(),
                                index,
                                index + character.len_utf8(),
                                path.clone(),
                            )
                            .unwrap(),
                        });
                    }
                    mem::swap(&mut parent, &mut token_trees);
                    let start_index = open_index + open_delimiter.as_open_char().len_utf8();
                    let full_span =
                        Span::new(src.clone(), start_index, index, path.clone()).unwrap();
                    let group = CommentedGroup {
                        token_stream: CommentedTokenStream {
                            token_trees: parent,
                            full_span,
                        },
                        delimiter: close_delimiter,
                        span: span_until(src, open_index, &mut char_indices, &path),
                    };
                    token_trees.push(CommentedTokenTree::Tree(group.into()));
                }
            }
            continue;
        }
        if character == '"' {
            let mut parsed = String::new();
            loop {
                let next_character = match char_indices.next() {
                    Some((_, next_character)) => next_character,
                    None => {
                        return Err(LexError {
                            kind: LexErrorKind::UnclosedStringLiteral { position: index },
                            span: Span::new(src.clone(), index, src.len() - 1, path.clone())
                                .unwrap(),
                        });
                    }
                };
                match next_character {
                    '\\' => {
                        let parsed_character =
                            match parse_escape_code(src, &mut char_indices, &path) {
                                Ok(parsed_character) => parsed_character,
                                Err(None) => {
                                    return Err(LexError {
                                        kind: LexErrorKind::UnclosedStringLiteral {
                                            position: index,
                                        },
                                        span: Span::new(
                                            src.clone(),
                                            index,
                                            src.len(),
                                            path.clone(),
                                        )
                                        .unwrap(),
                                    });
                                }
                                Err(Some(err)) => return Err(err),
                            };
                        parsed.push(parsed_character);
                    }
                    '"' => break,
                    _ => {
                        parsed.push(next_character);
                    }
                }
            }
            let span = span_until(src, index, &mut char_indices, &path);
            let literal = Literal::String(LitString { span, parsed });
            token_trees.push(CommentedTokenTree::Tree(literal.into()));
            continue;
        }
        if character == '\'' {
            let next_character = match char_indices.next() {
                Some((_, next_character)) => next_character,
                None => {
                    return Err(LexError {
                        kind: LexErrorKind::UnclosedCharLiteral { position: index },
                        span: Span::new(src.clone(), index, src.len(), path.clone()).unwrap(),
                    });
                }
            };
            let parsed = if next_character == '\\' {
                match parse_escape_code(src, &mut char_indices, &path) {
                    Ok(parsed) => parsed,
                    Err(None) => {
                        return Err(LexError {
                            kind: LexErrorKind::UnclosedCharLiteral { position: index },
                            span: Span::new(src.clone(), index, src.len(), path.clone()).unwrap(),
                        });
                    }
                    Err(Some(err)) => return Err(err),
                }
            } else {
                next_character
            };
            match char_indices.next() {
                None => {
                    return Err(LexError {
                        kind: LexErrorKind::UnclosedCharLiteral { position: index },
                        span: Span::new(src.clone(), index, src.len(), path.clone()).unwrap(),
                    });
                }
                Some((_, '\'')) => (),
                Some((next_index, unexpected_char)) => {
                    return Err(LexError {
                        kind: LexErrorKind::ExpectedCloseQuote {
                            position: next_index,
                        },
                        span: Span::new(
                            src.clone(),
                            next_index,
                            next_index + unexpected_char.len_utf8(),
                            path.clone(),
                        )
                        .unwrap(),
                    });
                }
            }
            let span = span_until(src, index, &mut char_indices, &path);
            let literal = Literal::Char(LitChar { span, parsed });
            token_trees.push(CommentedTokenTree::Tree(literal.into()));
            continue;
        }
        if let Some(digit) = character.to_digit(10) {
            let (big_uint, end_opt) = if digit == 0 {
                match char_indices.peek() {
                    Some((_, 'x')) => {
                        let _ = char_indices.next();
                        let (hex_digit_position, hex_digit) = match char_indices.next() {
                            None => {
                                return Err(LexError {
                                    kind: LexErrorKind::IncompleteHexIntLiteral { position: index },
                                    span: Span::new(src.clone(), index, src.len(), path.clone())
                                        .unwrap(),
                                });
                            }
                            Some((hex_digit_position, hex_digit)) => {
                                (hex_digit_position, hex_digit)
                            }
                        };
                        let hex_digit = match hex_digit.to_digit(16) {
                            Some(hex_digit) => hex_digit,
                            None => {
                                return Err(LexError {
                                    kind: LexErrorKind::IncompleteHexIntLiteral { position: index },
                                    span: Span::new(
                                        src.clone(),
                                        index,
                                        hex_digit_position,
                                        path.clone(),
                                    )
                                    .unwrap(),
                                });
                            }
                        };
                        let mut big_uint = BigUint::from(hex_digit);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 16);
                        (big_uint, end_opt)
                    }
                    Some((_, 'b')) => {
                        let _ = char_indices.next();
                        let (bin_digit_position, bin_digit) = match char_indices.next() {
                            None => {
                                return Err(LexError {
                                    kind: LexErrorKind::IncompleteBinaryIntLiteral {
                                        position: index,
                                    },
                                    span: Span::new(src.clone(), index, src.len(), path.clone())
                                        .unwrap(),
                                });
                            }
                            Some((bin_digit_position, bin_digit)) => {
                                (bin_digit_position, bin_digit)
                            }
                        };
                        let bin_digit = match bin_digit.to_digit(2) {
                            Some(bin_digit) => bin_digit,
                            None => {
                                return Err(LexError {
                                    kind: LexErrorKind::IncompleteBinaryIntLiteral {
                                        position: index,
                                    },
                                    span: Span::new(
                                        src.clone(),
                                        index,
                                        bin_digit_position,
                                        path.clone(),
                                    )
                                    .unwrap(),
                                });
                            }
                        };
                        let mut big_uint = BigUint::from(bin_digit);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 2);
                        (big_uint, end_opt)
                    }
                    Some((_, 'o')) => {
                        let _ = char_indices.next();
                        let (oct_digit_position, oct_digit) = match char_indices.next() {
                            None => {
                                return Err(LexError {
                                    kind: LexErrorKind::IncompleteOctalIntLiteral {
                                        position: index,
                                    },
                                    span: Span::new(src.clone(), index, src.len(), path.clone())
                                        .unwrap(),
                                });
                            }
                            Some((oct_digit_position, oct_digit)) => {
                                (oct_digit_position, oct_digit)
                            }
                        };
                        let oct_digit = match oct_digit.to_digit(2) {
                            Some(oct_digit) => oct_digit,
                            None => {
                                return Err(LexError {
                                    kind: LexErrorKind::IncompleteOctalIntLiteral {
                                        position: index,
                                    },
                                    span: Span::new(
                                        src.clone(),
                                        index,
                                        oct_digit_position,
                                        path.clone(),
                                    )
                                    .unwrap(),
                                });
                            }
                        };
                        let mut big_uint = BigUint::from(oct_digit);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 8);
                        (big_uint, end_opt)
                    }
                    Some((_, '_')) | Some((_, '0'..='9')) => {
                        let mut big_uint = BigUint::from(0u32);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 10);
                        (big_uint, end_opt)
                    }
                    Some(&(next_index, _)) => (BigUint::from(0u32), Some(next_index)),
                    None => (BigUint::from(0u32), None),
                }
            } else {
                let mut big_uint = BigUint::from(digit);
                let end_opt = parse_digits(&mut big_uint, &mut char_indices, 10);
                (big_uint, end_opt)
            };
            let end = end_opt.unwrap_or(src.len());
            let span = Span::new(src.clone(), index, end, path.clone()).unwrap();
            let ty_opt = match char_indices.peek() {
                Some((_, c)) if c.is_xid_continue() => {
                    let (suffix_start_position, c) = char_indices.next().unwrap();
                    let mut suffix = String::from(c);
                    let suffix_end_position = loop {
                        match char_indices.peek() {
                            Some((position, c)) => {
                                if c.is_xid_continue() {
                                    suffix.push(*c);
                                    let _ = char_indices.next();
                                } else {
                                    break *position;
                                }
                            }
                            None => break src.len(),
                        }
                    };
                    let ty = match &suffix[..] {
                        "u8" => LitIntType::U8,
                        "u16" => LitIntType::U16,
                        "u32" => LitIntType::U32,
                        "u64" => LitIntType::U64,
                        "i8" => LitIntType::I8,
                        "i16" => LitIntType::I16,
                        "i32" => LitIntType::I32,
                        "i64" => LitIntType::I64,
                        _ => {
                            let span = Span::new(
                                src.clone(),
                                suffix_start_position,
                                suffix_end_position,
                                path.clone(),
                            )
                            .unwrap();
                            return Err(LexError {
                                kind: LexErrorKind::InvalidIntSuffix {
                                    suffix: Ident::new(span.clone()),
                                },
                                span,
                            });
                        }
                    };
                    let span = span_until(src, suffix_start_position, &mut char_indices, &path);
                    Some((ty, span))
                }
                _ => None,
            };
            let literal = Literal::Int(LitInt {
                span,
                parsed: big_uint,
                ty_opt,
            });
            token_trees.push(CommentedTokenTree::Tree(literal.into()));
            continue;
        }
        if let Some(kind) = character.as_punct_kind() {
            let spacing = match char_indices.peek() {
                Some((_, next_character)) if next_character.as_punct_kind().is_some() => {
                    Spacing::Joint
                }
                _ => Spacing::Alone,
            };
            let span = span_until(src, index, &mut char_indices, &path);
            let punct = Punct {
                kind,
                spacing,
                span,
            };
            token_trees.push(CommentedTokenTree::Tree(punct.into()));
            continue;
        }
        return Err(LexError {
            kind: LexErrorKind::InvalidCharacter {
                position: index,
                character,
            },
            span: Span::new(
                src.clone(),
                index,
                index + character.len_utf8(),
                path.clone(),
            )
            .unwrap(),
        });
    }
    if let Some((_, open_position, open_delimiter)) = parent_token_trees.pop() {
        return Err(LexError {
            kind: LexErrorKind::UnclosedDelimiter {
                open_position,
                open_delimiter,
            },
            span: Span::new(
                src.clone(),
                open_position,
                open_position + open_delimiter.as_open_char().len_utf8(),
                path.clone(),
            )
            .unwrap(),
        });
    }
    let full_span = Span::new(src.clone(), start, end, path).unwrap();
    let token_stream = CommentedTokenStream {
        token_trees,
        full_span,
    };
    Ok(token_stream)
}

fn parse_escape_code(
    src: &Arc<str>,
    char_indices: &mut CharIndices,
    path: &Option<Arc<PathBuf>>,
) -> Result<char, Option<LexError>> {
    match char_indices.next() {
        None => Err(None),
        Some((_, '"')) => Ok('"'),
        Some((_, '\'')) => Ok('\''),
        Some((_, 'n')) => Ok('\n'),
        Some((_, 'r')) => Ok('\r'),
        Some((_, 't')) => Ok('\t'),
        Some((_, '\\')) => Ok('\\'),
        Some((_, '0')) => Ok('\0'),
        Some((index, 'x')) => {
            let (high, low) = match (char_indices.next(), char_indices.next()) {
                (Some((_, high)), Some((_, low))) => (high, low),
                _ => return Err(None),
            };
            let (high, low) = match (high.to_digit(16), low.to_digit(16)) {
                (Some(high), Some(low)) => (high, low),
                _ => {
                    let span = span_until(src, index, char_indices, path);
                    return Err(Some(LexError {
                        kind: LexErrorKind::InvalidHexEscape,
                        span,
                    }));
                }
            };
            let parsed_character = char::from_u32((high << 4) | low).unwrap();
            Ok(parsed_character)
        }
        Some((index, 'u')) => {
            match char_indices.next() {
                None => return Err(None),
                Some((_, '{')) => (),
                Some((_, unexpected_char)) => {
                    return Err(Some(LexError {
                        kind: LexErrorKind::UnicodeEscapeMissingBrace { position: index },
                        span: Span::new(
                            src.clone(),
                            index,
                            index + unexpected_char.len_utf8(),
                            path.clone(),
                        )
                        .unwrap(),
                    }));
                }
            }
            let mut digits_start_position_opt = None;
            let mut char_value = BigUint::from(0u32);
            let digits_end_position = loop {
                let (position, digit) = match char_indices.next() {
                    None => return Err(None),
                    Some((position, '}')) => break position,
                    Some((position, digit)) => (position, digit),
                };
                if digits_start_position_opt.is_none() {
                    digits_start_position_opt = Some(position);
                };
                let digit = match digit.to_digit(16) {
                    None => {
                        return Err(Some(LexError {
                            kind: LexErrorKind::InvalidUnicodeEscapeDigit { position },
                            span: Span::new(
                                src.clone(),
                                position,
                                position + digit.len_utf8(),
                                path.clone(),
                            )
                            .unwrap(),
                        }));
                    }
                    Some(digit) => digit,
                };
                char_value *= 16u32;
                char_value += digit;
            };
            let digits_start_position = digits_start_position_opt.unwrap_or(digits_end_position);
            let char_value = match u32::try_from(char_value) {
                Err(..) => {
                    return Err(Some(LexError {
                        kind: LexErrorKind::UnicodeEscapeOutOfRange { position: index },
                        span: Span::new(
                            src.clone(),
                            digits_start_position,
                            digits_end_position,
                            path.clone(),
                        )
                        .unwrap(),
                    }));
                }
                Ok(char_value) => char_value,
            };
            let parsed_character = match char::from_u32(char_value) {
                None => {
                    let span = span_until(src, index, char_indices, path);
                    return Err(Some(LexError {
                        kind: LexErrorKind::UnicodeEscapeInvalidCharValue { span },
                        span: Span::new(
                            src.clone(),
                            digits_start_position,
                            digits_end_position,
                            path.clone(),
                        )
                        .unwrap(),
                    }));
                }
                Some(parsed_character) => parsed_character,
            };
            Ok(parsed_character)
        }
        Some((index, unexpected_char)) => Err(Some(LexError {
            kind: LexErrorKind::InvalidEscapeCode { position: index },
            span: Span::new(
                src.clone(),
                index,
                index + unexpected_char.len_utf8(),
                path.clone(),
            )
            .unwrap(),
        })),
    }
}

fn parse_digits(
    big_uint: &mut BigUint,
    char_indices: &mut CharIndices,
    radix: u32,
) -> Option<usize> {
    loop {
        match char_indices.peek() {
            None => break None,
            Some((_, '_')) => {
                let _ = char_indices.next();
            }
            Some(&(index, character)) => match character.to_digit(radix) {
                None => break Some(index),
                Some(digit) => {
                    let _ = char_indices.next();
                    *big_uint *= radix;
                    *big_uint += digit;
                }
            },
        };
    }
}

fn span_until(
    src: &Arc<str>,
    start: usize,
    char_indices: &mut CharIndices,
    path: &Option<Arc<PathBuf>>,
) -> Span {
    let end = match char_indices.peek() {
        Some(&(end, _)) => end,
        None => src.len(),
    };
    Span::new(src.clone(), start, end, path.clone()).unwrap()
}

impl TokenStream {
    pub fn token_trees(&self) -> &[TokenTree] {
        &self.token_trees
    }
}

impl Spanned for TokenStream {
    fn span(&self) -> Span {
        self.full_span.clone()
    }
}

impl CommentedTokenTree {
    pub fn strip_comments(self) -> Option<TokenTree> {
        let commented_tt = match self {
            Self::Comment(_) => return None,
            Self::Tree(commented_tt) => commented_tt,
        };
        let tt = match commented_tt {
            CommentedTree::Punct(punct) => punct.into(),
            CommentedTree::Ident(ident) => ident.into(),
            CommentedTree::Group(group) => group.strip_comments().into(),
            CommentedTree::Literal(lit) => lit.into(),
        };
        Some(tt)
    }
}

impl CommentedTokenStream {
    pub fn token_trees(&self) -> &[CommentedTokenTree] {
        &self.token_trees
    }

    pub fn strip_comments(self) -> TokenStream {
        let token_trees = self
            .token_trees
            .into_iter()
            .filter_map(|tree| tree.strip_comments())
            .collect();
        TokenStream {
            token_trees,
            full_span: self.full_span,
        }
    }
}

impl Spanned for CommentedTokenStream {
    fn span(&self) -> Span {
        self.full_span.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{lex_commented, CommentedTokenTree, CommentedTree};
    use crate::priv_prelude::*;
    use std::sync::Arc;

    #[test]
    fn lex_commented_token_stream() {
        let input = r#"
        // Single-line comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let start = 0;
        let end = input.len();
        let path = None;
        let stream = lex_commented(&Arc::from(input), start, end, path).unwrap();
        let mut tts = stream.token_trees().iter();
        assert_eq!(
            tts.next().unwrap().span().as_str(),
            "// Single-line comment."
        );
        assert_eq!(tts.next().unwrap().span().as_str(), "struct");
        assert_eq!(tts.next().unwrap().span().as_str(), "Foo");
        {
            let group = match tts.next() {
                Some(CommentedTokenTree::Tree(CommentedTree::Group(group))) => group,
                _ => panic!("expected group"),
            };
            let mut tts = group.token_stream.token_trees().iter();
            assert_eq!(
                tts.next().unwrap().span().as_str(),
                "/* multi-\n             * line-\n             * comment */",
            );
            assert_eq!(tts.next().unwrap().span().as_str(), "bar");
            assert_eq!(tts.next().unwrap().span().as_str(), ":");
            assert_eq!(tts.next().unwrap().span().as_str(), "i32");
            assert_eq!(tts.next().unwrap().span().as_str(), ",");
            assert!(tts.next().is_none());
        }
        assert!(tts.next().is_none());
    }
}
