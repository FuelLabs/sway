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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Punct {
    pub span: Span,
    pub kind: PunctKind,
    pub spacing: Spacing,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group {
    pub delimiter: Delimiter,
    pub token_stream: TokenStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenTree {
    Punct(Punct),
    Ident(Ident),
    Group(Group),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenStream {
    token_trees: Vec<TokenTree>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LexError {
    UnclosedMultilineComment {
        unclosed_indices: Vec<usize>,
    },
    UnexpectedCloseDelimiter {
        position: usize,
        close_delimiter: Delimiter,
    },
    MismatchedDelimiters {
        open_position: usize,
        close_position: usize,
        open_delimiter: Delimiter,
        close_delimiter: Delimiter,
    },
    UnclosedDelimiter {
        open_position: usize,
        open_delimiter: Delimiter,
    },
    UnclosedStringLiteral {
        position: usize,
    },
    UnclosedCharLiteral {
        position: usize,
    },
    ExpectedCloseQuote {
        position: usize,
    },
    IncompleteHexIntLiteral {
        position: usize,
    },
    IncompleteBinaryIntLiteral {
        position: usize,
    },
    IncompleteOctalIntLiteral {
        position: usize,
    },
    InvalidCharacter {
        position: usize,
        character: char,
    },
    InvalidHexEscape {
        span: Span,
    },
    UnicodeEscapeMissingBrace {
        position: usize,
    },
    InvalidUnicodeEscapeDigit {
        position: usize,
    },
    UnicodeEscapeOutOfRange {
        position: usize,
    },
    UnicodeEscapeInvalidCharValue {
        span: Span,
    },
    InvalidEscapeCode {
        position: usize,
    },
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
            _ => None,
        }
    }
}

type CharIndices<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

pub fn lex(src: &Arc<str>) -> Result<TokenStream, LexError> {
    let mut char_indices = src.char_indices().peekable();
    let mut parent_token_trees = Vec::new();
    let mut token_trees = Vec::new();
    loop {
        let (index, character) = match char_indices.next() {
            Some((index, character)) => (index, character),
            None => break,
        };
        if character.is_whitespace() {
            continue;
        }
        if character == '/' {
            match char_indices.peek() {
                Some((_, '/')) => {
                    let _ = char_indices.next();
                    loop {
                        let character = match char_indices.next() {
                            Some((_, next_character)) => next_character,
                            None => break,
                        };
                        if character == '\n' {
                            break;
                        }
                    }
                },
                Some((_, '*')) => {
                    let _ = char_indices.next();
                    let mut unclosed_indices = vec![index];
                    loop {
                        match char_indices.next() {
                            None => {
                                return Err(LexError::UnclosedMultilineComment {
                                    unclosed_indices,
                                })
                            },
                            Some((_, '*')) => match char_indices.next() {
                                None => {
                                    return Err(LexError::UnclosedMultilineComment {
                                        unclosed_indices,
                                    })
                                },
                                Some((_, '/')) => {
                                    let _ = char_indices.next();
                                    unclosed_indices.pop();
                                    if unclosed_indices.is_empty() {
                                        break;
                                    }
                                },
                                Some((_, _)) => (),
                            },
                            Some((next_index, '/')) => match char_indices.next() {
                                None => {
                                    return Err(LexError::UnclosedMultilineComment {
                                        unclosed_indices,
                                    })
                                },
                                Some((_, '*')) => {
                                    unclosed_indices.push(next_index);
                                },
                                Some((_, _)) => (),
                            },
                            Some((_, _)) => (),
                        }
                    }
                },
                Some(&(end, next_character)) => {
                    let spacing = if let Some(..) = next_character.as_punct_kind() {
                        Spacing::Joint
                    } else {
                        Spacing::Alone
                    };
                    let span = Span {
                        src: src.clone(),
                        start: index,
                        end,
                    };
                    let punct = Punct {
                        kind: PunctKind::ForwardSlash,
                        spacing,
                        span,
                    };
                    token_trees.push(TokenTree::Punct(punct));
                },
                None => {
                    let span = Span {
                        src: src.clone(),
                        start: index,
                        end: src.len(),
                    };
                    let punct = Punct {
                        kind: PunctKind::ForwardSlash,
                        spacing: Spacing::Alone,
                        span,
                    };
                    token_trees.push(TokenTree::Punct(punct));
                },
            }
            continue;
        }
        if character.is_xid_start() {
            loop {
                let next_character = match char_indices.peek() {
                    Some((_, next_character)) => next_character,
                    None => break,
                };
                if !next_character.is_xid_continue() {
                    break;
                }
                let _ = char_indices.next();
            };
            let span = span_until(src, index, &mut char_indices);
            let ident = Ident { span };
            token_trees.push(TokenTree::Ident(ident));
            continue;
        }
        if let Some(delimiter) = character.as_open_delimiter() {
            let token_trees = mem::replace(&mut token_trees, Vec::new());
            parent_token_trees.push((token_trees, index, delimiter));
            continue;
        }
        if let Some(close_delimiter) = character.as_close_delimiter() {
            match parent_token_trees.pop() {
                None => {
                    return Err(LexError::UnexpectedCloseDelimiter {
                        position: index,
                        close_delimiter,
                    })
                },
                Some((mut parent, open_index, open_delimiter)) => {
                    if open_delimiter != close_delimiter {
                        return Err(LexError::MismatchedDelimiters {
                            open_position: open_index,
                            close_position: index,
                            open_delimiter,
                            close_delimiter,
                        })
                    }
                    mem::swap(&mut parent, &mut token_trees);
                    let group = Group {
                        token_stream: TokenStream {
                            token_trees: parent,
                        },
                        delimiter: close_delimiter,
                    };
                    token_trees.push(TokenTree::Group(group));
                },
            }
            continue;
        }
        if character == '"' {
            let mut parsed = String::new();
            loop {
                let next_character = match char_indices.next() {
                    Some((_, next_character)) => next_character,
                    None => {
                        return Err(LexError::UnclosedStringLiteral {
                            position: index,
                        })
                    },
                };
                match next_character {
                    '\\' => {
                        let parsed_character = match parse_escape_code(src, &mut char_indices) {
                            Ok(parsed_character) => parsed_character,
                            Err(None) => {
                                return Err(LexError::UnclosedStringLiteral {
                                    position: index,
                                });
                            },
                            Err(Some(err)) => return Err(err),
                        };
                        parsed.push(parsed_character);
                    },
                    '"' => break,
                    _ => {
                        parsed.push(next_character);
                    },
                }
            }
            let span = span_until(src, index, &mut char_indices);
            let literal = Literal::String(LitString { span, parsed });
            token_trees.push(TokenTree::Literal(literal));
            continue;
        }
        if character == '\'' {
            let next_character = match char_indices.next() {
                Some((_, next_character)) => next_character,
                None => {
                    return Err(LexError::UnclosedCharLiteral {
                        position: index,
                    })
                },
            };
            let parsed = if next_character == '\\' {
                match parse_escape_code(src, &mut char_indices) {
                    Ok(parsed) => parsed,
                    Err(None) => {
                        return Err(LexError::UnclosedCharLiteral {
                            position: index,
                        });
                    },
                    Err(Some(err)) => return Err(err),
                }
            } else {
                next_character
            };
            match char_indices.next() {
                None => {
                    return Err(LexError::UnclosedCharLiteral {
                        position: index,
                    })
                },
                Some((_, '\'')) => (),
                Some((next_index, _)) => {
                    return Err(LexError::ExpectedCloseQuote {
                        position: next_index,
                    })
                },
            }
            let span = span_until(src, index, &mut char_indices);
            let literal = Literal::Char(LitChar { span, parsed });
            token_trees.push(TokenTree::Literal(literal));
            continue;
        }
        if let Some(digit) = character.to_digit(10) {
            let (big_uint, end_opt) = if digit == 0 {
                match char_indices.peek() {
                    Some((_, 'x')) => {
                        let _ = char_indices.next();
                        let hex_digit = match char_indices.next() {
                            None => {
                                return Err(LexError::IncompleteHexIntLiteral {
                                    position: index,
                                });
                            },
                            Some((_, hex_digit)) => hex_digit,
                        };
                        let hex_digit = match hex_digit.to_digit(16) {
                            Some(hex_digit) => hex_digit,
                            None => {
                                return Err(LexError::IncompleteHexIntLiteral {
                                    position: index,
                                });
                            },
                        };
                        let mut big_uint = BigUint::from(hex_digit);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 16);
                        (big_uint, end_opt)
                    },
                    Some((_, 'b')) => {
                        let _ = char_indices.next();
                        let bin_digit = match char_indices.next() {
                            None => {
                                return Err(LexError::IncompleteBinaryIntLiteral {
                                    position: index,
                                });
                            },
                            Some((_, bin_digit)) => bin_digit,
                        };
                        let bin_digit = match bin_digit.to_digit(2) {
                            Some(bin_digit) => bin_digit,
                            None => {
                                return Err(LexError::IncompleteBinaryIntLiteral {
                                    position: index,
                                });
                            },
                        };
                        let mut big_uint = BigUint::from(bin_digit);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 2);
                        (big_uint, end_opt)
                    },
                    Some((_, 'o')) => {
                        let _ = char_indices.next();
                        let oct_digit = match char_indices.next() {
                            None => {
                                return Err(LexError::IncompleteOctalIntLiteral {
                                    position: index,
                                });
                            },
                            Some((_, oct_digit)) => oct_digit,
                        };
                        let oct_digit = match oct_digit.to_digit(2) {
                            Some(oct_digit) => oct_digit,
                            None => {
                                return Err(LexError::IncompleteOctalIntLiteral {
                                    position: index,
                                });
                            },
                        };
                        let mut big_uint = BigUint::from(oct_digit);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 8);
                        (big_uint, end_opt)
                    },
                    Some((_, '_')) | Some((_, '0'..='9')) => {
                        let mut big_uint = BigUint::from(0u32);
                        let end_opt = parse_digits(&mut big_uint, &mut char_indices, 10);
                        (big_uint, end_opt)
                    },
                    Some(&(next_index, _)) => (BigUint::from(0u32), Some(next_index)),
                    None => (BigUint::from(0u32), None),
                }
            } else {
                let mut big_uint = BigUint::from(digit);
                let end_opt = parse_digits(&mut big_uint, &mut char_indices, 10);
                (big_uint, end_opt)
            };
            let end = end_opt.unwrap_or_else(|| src.len());
            let span = Span {
                src: src.clone(),
                start: index,
                end,
            };
            let ty_opt = match char_indices.peek() {
                Some((_, c)) if c.is_xid_continue() => {
                    let (suffix_start_position, c) = char_indices.next().unwrap();
                    let mut suffix = String::from(c);
                    loop {
                        match char_indices.peek() {
                            Some((_, c)) if c.is_xid_continue() => {
                                suffix.push(*c);
                                let _ = char_indices.next();
                            },
                            _ => break,
                        }
                    }
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
                            return Err(LexError::InvalidCharacter {
                                position: index,
                                character,
                            });
                        },
                    };
                    let span = span_until(src, suffix_start_position, &mut char_indices);
                    Some((ty, span))
                },
                _ => None,
            };
            let literal = Literal::Int(LitInt { span, parsed: big_uint, ty_opt });
            token_trees.push(TokenTree::Literal(literal));
            continue;
        }
        if let Some(kind) = character.as_punct_kind() {
            let spacing = match char_indices.peek() {
                Some((_, next_character)) if next_character.as_punct_kind().is_some() => {
                    Spacing::Joint
                },
                _ => Spacing::Alone,
            };
            let span = span_until(src, index, &mut char_indices);
            let punct = Punct {
                kind,
                spacing,
                span,
            };
            token_trees.push(TokenTree::Punct(punct));
            continue;
        }
        return Err(LexError::InvalidCharacter {
            position: index,
            character,
        });
    }
    if let Some((_, open_position, open_delimiter)) = parent_token_trees.pop() {
        return Err(LexError::UnclosedDelimiter {
            open_position,
            open_delimiter,
        });
    }
    let token_stream = TokenStream { token_trees };
    Ok(token_stream)
}

fn parse_escape_code(src: &Arc<str>, char_indices: &mut CharIndices) -> Result<char, Option<LexError>> {
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
                    let span = span_until(src, index, char_indices);
                    return Err(Some(LexError::InvalidHexEscape { span }));
                },
            };
            let parsed_character = char::from_u32((high << 4) | low).unwrap();
            Ok(parsed_character)
        },
        Some((index, 'u')) => {
            match char_indices.next() {
                None => return Err(None),
                Some((_, '{')) => (),
                Some((_, _)) => {
                    return Err(Some(LexError::UnicodeEscapeMissingBrace {
                        position: index,
                    }));
                },
            }
            let mut char_value = 0u32;
            loop {
                let (position, digit) = match char_indices.next() {
                    None => return Err(None),
                    Some((_, '}')) => break,
                    Some((position, digit)) => (position, digit),
                };
                let digit = match digit.to_digit(16) {
                    None => {
                        return Err(Some(LexError::InvalidUnicodeEscapeDigit {
                            position,
                        }));
                    },
                    Some(digit) => digit,
                };
                match char_value.checked_mul(16) {
                    None => {
                        return Err(Some(LexError::UnicodeEscapeOutOfRange {
                            position: index,
                        }));
                    },
                    Some(new_char_value) => {
                        char_value = new_char_value | digit;
                    },
                }
            }
            let parsed_character = match char::from_u32(char_value) {
                None => {
                    let span = span_until(src, index, char_indices);
                    return Err(Some(LexError::UnicodeEscapeInvalidCharValue { span }));
                },
                Some(parsed_character) => parsed_character,
            };
            Ok(parsed_character)
        },
        Some((index, _)) => {
            Err(Some(LexError::InvalidEscapeCode {
                position: index,
            }))
        },
    }
}

fn parse_digits(big_uint: &mut BigUint, char_indices: &mut CharIndices, radix: u32) -> Option<usize> {
    loop {
        match char_indices.peek() {
            None => break None,
            Some((_, '_')) => {
                let _ = char_indices.next();
            },
            Some(&(index, character)) => {
                match character.to_digit(radix) {
                    None => break Some(index),
                    Some(digit) => {
                        let _ = char_indices.next();
                        *big_uint *= radix;
                        *big_uint += digit;
                    },
                }
            },
        };
    }
}

fn span_until(src: &Arc<str>, start: usize, char_indices: &mut CharIndices) -> Span {
    let end = match char_indices.peek() {
        Some(&(end, _)) => end,
        None => src.len(),
    };
    Span {
        src: src.clone(),
        start,
        end,
    }
}

impl TokenStream {
    pub fn token_trees(&self) -> &[TokenTree] {
        &self.token_trees
    }
}

