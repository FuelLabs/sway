use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Op(String),
    Ctrl(char),
    Ident(String),
    Fn,
    Let,
    Print,
    If,
    Else,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            Token::Bool(x) => write!(f, "{}", x),
            Token::Num(n) => write!(f, "{}", n),
            Token::Str(s) => write!(f, "{}", s),
            Token::Op(s) => write!(f, "{}", s),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Print => write!(f, "print"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
        }
    }
}
