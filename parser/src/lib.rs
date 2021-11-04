mod lexer;
mod span;
mod token;

pub(crate) use lexer::*;
pub(crate) use span::*;
pub(crate) use token::*;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::{collections::HashMap, env, fmt, fs};

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Func(String),
}

impl Value {
    fn num(self, span: Span) -> Result<f64, Error> {
        if let Value::Num(x) = self {
            Ok(x)
        } else {
            Err(Error {
                span,
                msg: format!("'{}' is not a number", self),
            })
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Num(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::List(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Func(name) => write!(f, "<function: {}>", name),
        }
    }
}

#[derive(Clone, Debug)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
}

pub type Spanned<T> = (T, Span);

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
enum Expr {
    Error,
    Value(Value),
    List(Vec<Spanned<Self>>),
    Local(String),
    Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Spanned<Vec<Spanned<Self>>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Print(Box<Spanned<Self>>),
}

// A function node in the AST.
#[derive(Debug)]
struct Func {
    args: Vec<String>,
    body: Spanned<Expr>,
}

fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let raw_expr = recursive(|raw_expr| {
            let val = filter_map(|span, tok| match tok {
                Token::Null => Ok(Expr::Value(Value::Null)),
                Token::Bool(x) => Ok(Expr::Value(Value::Bool(x))),
                Token::Num(n) => Ok(Expr::Value(Value::Num(n.parse().unwrap()))),
                Token::Str(s) => Ok(Expr::Value(Value::Str(s))),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            })
            .labelled("value");

            let ident = filter_map(|span, tok| match tok {
                Token::Ident(ident) => Ok(ident.clone()),
                _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
            })
            .labelled("identifier");

            // A list of expressions
            let items = expr
                .clone()
                .chain(just(Token::Ctrl(',')).ignore_then(expr.clone()).repeated())
                .then_ignore(just(Token::Ctrl(',')).or_not())
                .or_not()
                .map(|item| item.unwrap_or_else(Vec::new));

            // A let expression
            let let_ = just(Token::Let)
                .ignore_then(ident)
                .then_ignore(just(Token::Op("=".to_string())))
                .then(raw_expr)
                .then_ignore(just(Token::Ctrl(';')))
                .then(expr.clone())
                .map(|((name, val), body)| Expr::Let(name, Box::new(val), Box::new(body)));

            let list = items
                .clone()
                .delimited_by(Token::Ctrl('['), Token::Ctrl(']'))
                .map(Expr::List);

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(ident.map(Expr::Local))
                .or(let_)
                .or(list)
                // In Nano Rust, `print` is just a keyword, just like Python 2, for simplicity
                .or(just(Token::Print)
                    .ignore_then(
                        expr.clone()
                            .delimited_by(Token::Ctrl('('), Token::Ctrl(')')),
                    )
                    .map(|expr| Expr::Print(Box::new(expr))))
                .map_with_span(|expr, span| (expr, span))
                // Atoms can also just be normal expressions, but surrounded with parentheses
                .or(expr
                    .clone()
                    .delimited_by(Token::Ctrl('('), Token::Ctrl(')')))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('('),
                    Token::Ctrl(')'),
                    [
                        (Token::Ctrl('['), Token::Ctrl(']')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                ))
                // Attempt to recover anything that looks like a list but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('['),
                    Token::Ctrl(']'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| (Expr::Error, span),
                ));

            // Function calls have very high precedence so we prioritise them
            let call = atom
                .then(
                    items
                        .delimited_by(Token::Ctrl('('), Token::Ctrl(')'))
                        .map_with_span(|args, span| (args, span))
                        .repeated(),
                )
                .foldl(|f, args| {
                    let span = f.1.start..args.1.end;
                    (Expr::Call(Box::new(f), args), span)
                });

            // Product ops (multiply and divide) have equal precedence
            let op = just(Token::Op("*".to_string()))
                .to(BinaryOp::Mul)
                .or(just(Token::Op("/".to_string())).to(BinaryOp::Div));
            let product = call
                .clone()
                .then(op.then(call).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            // Sum ops (add and subtract) have equal precedence
            let op = just(Token::Op("+".to_string()))
                .to(BinaryOp::Add)
                .or(just(Token::Op("-".to_string())).to(BinaryOp::Sub));
            let sum = product
                .clone()
                .then(op.then(product).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            // Comparison ops (equal, not-equal) have equal precedence
            let op = just(Token::Op("==".to_string()))
                .to(BinaryOp::Eq)
                .or(just(Token::Op("!=".to_string())).to(BinaryOp::NotEq));
            let compare = sum
                .clone()
                .then(op.then(sum).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            compare
        });

        // Blocks are expressions but delimited with braces
        let block = expr
            .clone()
            .delimited_by(Token::Ctrl('{'), Token::Ctrl('}'))
            // Attempt to recover anything that looks like a block but contains errors
            .recover_with(nested_delimiters(
                Token::Ctrl('{'),
                Token::Ctrl('}'),
                [
                    (Token::Ctrl('('), Token::Ctrl(')')),
                    (Token::Ctrl('['), Token::Ctrl(']')),
                ],
                |span| (Expr::Error, span),
            ));

        let if_ = recursive(|if_| {
            just(Token::If)
                .ignore_then(expr.clone())
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(block.clone().or(if_))
                        .or_not(),
                )
                .map_with_span(|((cond, a), b), span| {
                    (
                        Expr::If(
                            Box::new(cond),
                            Box::new(a),
                            Box::new(match b {
                                Some(b) => b,
                                // If an `if` expression has no trailing `else` block, we magic up one that just produces null
                                None => (Expr::Value(Value::Null), span.clone()),
                            }),
                        ),
                        span,
                    )
                })
        });

        // Both blocks and `if` are 'block expressions' and can appear in the place of statements
        let block_expr = block.or(if_).labelled("block");

        let block_chain = block_expr
            .clone()
            .then(block_expr.clone().repeated())
            .foldl(|a, b| {
                let span = a.1.start..b.1.end;
                (Expr::Then(Box::new(a), Box::new(b)), span)
            });

        block_chain
            // Expressions, chained by semicolons, are statements
            .or(raw_expr.clone())
            .then(just(Token::Ctrl(';')).ignore_then(expr.or_not()).repeated())
            .foldl(|a, b| {
                let span = a.1.clone(); // TODO: Not correct
                (
                    Expr::Then(
                        Box::new(a),
                        Box::new(match b {
                            Some(b) => b,
                            None => (Expr::Value(Value::Null), span.clone()),
                        }),
                    ),
                    span,
                )
            })
    })
}

fn funcs_parser() -> impl Parser<Token, HashMap<String, Func>, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident.clone()),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    // Argument lists are just identifiers separated by commas, surrounded by parentheses
    let args = ident
        .clone()
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .delimited_by(Token::Ctrl('('), Token::Ctrl(')'))
        .labelled("function args");

    let func = just(Token::Fn)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then(args)
        .then(
            expr_parser()
                .delimited_by(Token::Ctrl('{'), Token::Ctrl('}'))
                // Attempt to recover anything that looks like a function body but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('{'),
                    Token::Ctrl('}'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('['), Token::Ctrl(']')),
                    ],
                    |span| (Expr::Error, span),
                )),
        )
        .map(|((name, args), body)| (name, Func { args, body }))
        .labelled("function");

    func.repeated()
        .try_map(|fs, _| {
            let mut funcs = HashMap::new();
            for ((name, name_span), f) in fs {
                if funcs.insert(name.clone(), f).is_some() {
                    return Err(Simple::custom(
                        name_span.clone(),
                        format!("Function '{}' already exists", name),
                    ));
                }
            }
            Ok(funcs)
        })
        .then_ignore(end())
}

struct Error {
    span: Span,
    msg: String,
}

fn eval_expr(
    expr: &Spanned<Expr>,
    funcs: &HashMap<String, Func>,
    stack: &mut Vec<(String, Value)>,
) -> Result<Value, Error> {
    Ok(match &expr.0 {
        Expr::Error => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST
        Expr::Value(val) => val.clone(),
        Expr::List(items) => Value::List(
            items
                .iter()
                .map(|item| eval_expr(item, funcs, stack))
                .collect::<Result<_, _>>()?,
        ),
        Expr::Local(name) => stack
            .iter()
            .rev()
            .find(|(l, _)| l == name)
            .map(|(_, v)| v.clone())
            .or_else(|| Some(Value::Func(name.clone())).filter(|_| funcs.contains_key(name)))
            .ok_or_else(|| Error {
                span: expr.1.clone(),
                msg: format!("No such variable '{}' in scope", name),
            })?,
        Expr::Let(local, val, body) => {
            let val = eval_expr(val, funcs, stack)?;
            stack.push((local.clone(), val));
            let res = eval_expr(body, funcs, stack)?;
            stack.pop();
            res
        }
        Expr::Then(a, b) => {
            eval_expr(a, funcs, stack)?;
            eval_expr(b, funcs, stack)?
        }
        Expr::Binary(a, BinaryOp::Add, b) => Value::Num(
            eval_expr(a, funcs, stack)?.num(a.1.clone())?
                + eval_expr(b, funcs, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Sub, b) => Value::Num(
            eval_expr(a, funcs, stack)?.num(a.1.clone())?
                - eval_expr(b, funcs, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Mul, b) => Value::Num(
            eval_expr(a, funcs, stack)?.num(a.1.clone())?
                * eval_expr(b, funcs, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Div, b) => Value::Num(
            eval_expr(a, funcs, stack)?.num(a.1.clone())?
                / eval_expr(b, funcs, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Eq, b) => {
            Value::Bool(eval_expr(a, funcs, stack)? == eval_expr(b, funcs, stack)?)
        }
        Expr::Binary(a, BinaryOp::NotEq, b) => {
            Value::Bool(eval_expr(a, funcs, stack)? != eval_expr(b, funcs, stack)?)
        }
        Expr::Call(func, (args, args_span)) => {
            let f = eval_expr(func, funcs, stack)?;
            match f {
                Value::Func(name) => {
                    let f = &funcs[&name];
                    let mut stack = if f.args.len() != args.len() {
                        return Err(Error {
                            span: args_span.clone(),
                            msg: format!("'{}' called with wrong number of arguments (expected {}, found {})", name, f.args.len(), args.len()),
                        });
                    } else {
                        f.args
                            .iter()
                            .zip(args.iter())
                            .map(|(name, arg)| Ok((name.clone(), eval_expr(arg, funcs, stack)?)))
                            .collect::<Result<_, _>>()?
                    };
                    eval_expr(&f.body, funcs, &mut stack)?
                }
                f => {
                    return Err(Error {
                        span: func.1.clone(),
                        msg: format!("'{:?}' is not callable", f),
                    })
                }
            }
        }
        Expr::If(cond, a, b) => {
            let c = eval_expr(cond, funcs, stack)?;
            match c {
                Value::Bool(true) => eval_expr(a, funcs, stack)?,
                Value::Bool(false) => eval_expr(b, funcs, stack)?,
                c => {
                    return Err(Error {
                        span: cond.1.clone(),
                        msg: format!("Conditions must be booleans, found '{:?}'", c),
                    })
                }
            }
        }
        Expr::Print(a) => {
            let val = eval_expr(a, funcs, stack)?;
            println!("{}", val);
            val
        }
    })
}

pub fn main() {
    let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
        .expect("Failed to read file");

    let (tokens, mut errs) = Lexer::lexer().parse_recovery(src.as_str());

    let parse_errs = if let Some(tokens) = tokens {
        // println!("Tokens = {:?}", tokens);
        let len = src.chars().count();
        let (ast, parse_errs) =
            funcs_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        println!("{:#?}", ast);
        if let Some(funcs) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            if let Some(main) = funcs.get("main") {
                assert_eq!(main.args.len(), 0);
                match eval_expr(&main.body, &funcs, &mut Vec::new()) {
                    Ok(val) => println!("Return value: {}", val),
                    Err(e) => errs.push(Simple::custom(e.span, e.msg)),
                }
            } else {
                panic!("No main function!");
            }
        }

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    )
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Must be closed before this {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token in input"
                        } else {
                            "Unexpected end of input"
                        },
                        if e.expected().len() == 0 {
                            "end of input".to_string()
                        } else {
                            e.expected()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Unexpected token {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&src)).unwrap();
        });
}
