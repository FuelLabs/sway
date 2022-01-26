pub use crate::priv_prelude::*;

pub enum Expr {
    Path(Path), 
    StringLiteral(StringLiteral),
    IntLiteral(IntLiteral),
    Tuple(ExprTuple),
    Array {
        elems: SquareBrackets<Punctuated<Expr, CommaToken>>,
    },
    ArrayRepeat(ExprArrayRepeat),
    Parens(Box<Parens<Expr>>),
    CodeBlock(CodeBlock),
    FuncApp {
        func: Box<Expr>,
        args: Parens<Punctuated<Expr, CommaToken>>,
    },
    Index {
        target: Box<Expr>,
        arg: SquareBrackets<Box<Expr>>,
    },
    MethodCall {
        target: Box<Expr>,
        dot_token: DotToken,
        name: Ident,
        args: Parens<Punctuated<Expr, CommaToken>>,
    },
    FieldProjection {
        target: Box<Expr>,
        dot_token: DotToken,
        name: Ident,
    },
    Not {
        bang_token: BangToken,
        expr: Box<Expr>,
    },
    Mul {
        lhs: Box<Expr>,
        star_token: StarToken,
        rhs: Box<Expr>,
    },
    Div {
        lhs: Box<Expr>,
        forward_slash_token: ForwardSlashToken,
        rhs: Box<Expr>,
    },
    Modulo {
        lhs: Box<Expr>,
        percent_token: PercentToken,
        rhs: Box<Expr>,
    },
    Add {
        lhs: Box<Expr>,
        add_token: AddToken,
        rhs: Box<Expr>,
    },
    Sub {
        lhs: Box<Expr>,
        sub_token: SubToken,
        rhs: Box<Expr>,
    },
    Shl {
        lhs: Box<Expr>,
        shl_token: ShlToken,
        rhs: Box<Expr>,
    },
    Shr {
        lhs: Box<Expr>,
        shr_token: ShrToken,
        rhs: Box<Expr>,
    },
    BitAnd {
        lhs: Box<Expr>,
        ampersand_token: AmpersandToken,
        rhs: Box<Expr>,
    },
    BitXor {
        lhs: Box<Expr>,
        caret_token: CaretToken,
        rhs: Box<Expr>,
    },
    BitOr {
        lhs: Box<Expr>,
        pipe_token: PipeToken,
        rhs: Box<Expr>,
    },
    Equal {
        lhs: Box<Expr>,
        double_eq_token: DoubleEqToken,
        rhs: Box<Expr>,
    },
    NotEqual {
        lhs: Box<Expr>,
        bang_eq_token: BangEqToken,
        rhs: Box<Expr>,
    },
    LessThan {
        lhs: Box<Expr>,
        less_than_token: LessThanToken,
        rhs: Box<Expr>,
    },
    GreaterThan {
        lhs: Box<Expr>,
        greater_than_token: GreaterThanToken,
        rhs: Box<Expr>,
    },
    LessThanEq {
        lhs: Box<Expr>,
        less_than_eq_token: LessThanEqToken,
        rhs: Box<Expr>,
    },
    GreaterThanEq {
        lhs: Box<Expr>,
        greater_than_eq_token: GreaterThanEqToken,
        rhs: Box<Expr>,
    },
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Path(path) => path.span(),
            Expr::StringLiteral(string_literal) => string_literal.span(),
            Expr::IntLiteral(int_literal) => int_literal.span(),
            Expr::Tuple(expr_tuple) => expr_tuple.span(),
            Expr::Array { elems } => elems.span(),
            Expr::ArrayRepeat(expr_array_repeat) => expr_array_repeat.span(),
            Expr::Parens(parens) => parens.span(),
            Expr::CodeBlock(code_block) => code_block.span(),
            Expr::FuncApp { func, args } => {
                Span::join(func.span(), args.span())
            },
            Expr::Index { target, arg } => {
                Span::join(target.span(), arg.span())
            },
            Expr::MethodCall { target, args, .. } => {
                Span::join(target.span(), args.span())
            },
            Expr::FieldProjection { target, name, .. } => {
                Span::join(target.span(), name.span())
            },
            Expr::Not { bang_token, expr } => {
                Span::join(bang_token.span(), expr.span())
            },
            Expr::Mul { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Div { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Modulo { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Add { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Sub { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Shl { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Shr { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::BitAnd { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::BitXor { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::BitOr { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::Equal { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::NotEqual { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::LessThan { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::GreaterThan { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::LessThanEq { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
            Expr::GreaterThanEq { lhs, rhs, .. } => {
                Span::join(lhs.span(), rhs.span())
            },
        }
    }
}

pub fn expr() -> impl Parser<Output = Expr> + Clone {
    expr_precedence_comparison()
}

pub fn expr_precedence_comparison() -> impl Parser<Output = Expr> + Clone {
    enum Op {
        Equal {
            double_eq_token: DoubleEqToken,
            rhs: Expr,
        },
        NotEqual {
            bang_eq_token: BangEqToken,
            rhs: Expr,
        },
        LessThan {
            less_than_token: LessThanToken,
            rhs: Expr,
        },
        GreaterThan {
            greater_than_token: GreaterThanToken,
            rhs: Expr,
        },
        LessThanEq {
            less_than_eq_token: LessThanEqToken,
            rhs: Expr,
        },
        GreaterThanEq {
            greater_than_eq_token: GreaterThanEqToken,
            rhs: Expr,
        },
    }

    impl Spanned for Op {
        fn span(&self) -> Span {
            match self {
                Op::Equal { double_eq_token, rhs } => {
                    Span::join(double_eq_token.span(), rhs.span())
                },
                Op::NotEqual { bang_eq_token, rhs } => {
                    Span::join(bang_eq_token.span(), rhs.span())
                },
                Op::LessThan { less_than_token, rhs } => {
                    Span::join(less_than_token.span(), rhs.span())
                },
                Op::GreaterThan { greater_than_token, rhs } => {
                    Span::join(greater_than_token.span(), rhs.span())
                },
                Op::LessThanEq { less_than_eq_token, rhs } => {
                    Span::join(less_than_eq_token.span(), rhs.span())
                },
                Op::GreaterThanEq { greater_than_eq_token, rhs } => {
                    Span::join(greater_than_eq_token.span(), rhs.span())
                },
            }
        }
    }

    let op = {
        let equal = {
            double_eq_token()
            .then_optional_whitespace()
            .then(expr_precedence_bit_or())
            .map(|(double_eq_token, rhs)| Op::Equal { double_eq_token, rhs })
        };
        let not_equal = {
            bang_eq_token()
            .then_optional_whitespace()
            .then(expr_precedence_bit_or())
            .map(|(bang_eq_token, rhs)| Op::NotEqual { bang_eq_token, rhs })
        };
        let less_than = {
            less_than_token()
            .then_optional_whitespace()
            .then(expr_precedence_bit_or())
            .map(|(less_than_token, rhs)| Op::LessThan { less_than_token, rhs })
        };
        let greater_than = {
            greater_than_token()
            .then_optional_whitespace()
            .then(expr_precedence_bit_or())
            .map(|(greater_than_token, rhs)| Op::GreaterThan { greater_than_token, rhs })
        };
        let less_than_eq = {
            less_than_eq_token()
            .then_optional_whitespace()
            .then(expr_precedence_bit_or())
            .map(|(less_than_eq_token, rhs)| Op::LessThanEq { less_than_eq_token, rhs })
        };
        let greater_than_eq = {
            greater_than_eq_token()
            .then_optional_whitespace()
            .then(expr_precedence_bit_or())
            .map(|(greater_than_eq_token, rhs)| Op::GreaterThanEq { greater_than_eq_token, rhs })
        };

        equal
        .or(not_equal)
        .or(less_than_eq)
        .or(greater_than_eq)
        .or(less_than)
        .or(greater_than)
    };

    expr_precedence_bit_or()
    .then(optional_leading_whitespace(op).optional())
    .map(|(lhs, op_res): (_, Result<_, _>)| match op_res.ok() {
        None => lhs,
        Some(Op::Equal { double_eq_token, rhs }) => Expr::Equal {
            lhs: Box::new(lhs),
            double_eq_token,
            rhs: Box::new(rhs),
        },
        Some(Op::NotEqual { bang_eq_token, rhs }) => Expr::NotEqual {
            lhs: Box::new(lhs),
            bang_eq_token,
            rhs: Box::new(rhs),
        },
        Some(Op::LessThan { less_than_token, rhs }) => Expr::LessThan {
            lhs: Box::new(lhs),
            less_than_token,
            rhs: Box::new(rhs),
        },
        Some(Op::GreaterThan { greater_than_token, rhs }) => Expr::GreaterThan {
            lhs: Box::new(lhs),
            greater_than_token,
            rhs: Box::new(rhs),
        },
        Some(Op::LessThanEq { less_than_eq_token, rhs }) => Expr::LessThanEq {
            lhs: Box::new(lhs),
            less_than_eq_token,
            rhs: Box::new(rhs),
        },
        Some(Op::GreaterThanEq { greater_than_eq_token, rhs }) => Expr::GreaterThanEq {
            lhs: Box::new(lhs),
            greater_than_eq_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_bit_or() -> impl Parser<Output = Expr> + Clone {
    let bit_or = {
        pipe_token()
        .then_optional_whitespace()
        .then(expr_precedence_bit_xor())
    };

    expr_precedence_bit_xor()
    .then(optional_leading_whitespace(bit_or).repeated())
    .map(|(lhs, ops_with_span): (_, WithSpan<_>)| {
        let mut expr = lhs;
        for (pipe_token, rhs) in ops_with_span.parsed {
            expr = Expr::BitOr {
                lhs: Box::new(expr),
                pipe_token,
                rhs: Box::new(rhs),
            };
        }
        expr
    })
}

pub fn expr_precedence_bit_xor() -> impl Parser<Output = Expr> + Clone {
    let bit_xor = {
        caret_token()
        .then_optional_whitespace()
        .then(expr_precedence_bit_and())
    };

    expr_precedence_bit_and()
    .then(optional_leading_whitespace(bit_xor).repeated())
    .map(|(lhs, ops_with_span): (_, WithSpan<_>)| {
        let mut expr = lhs;
        for (caret_token, rhs) in ops_with_span.parsed {
            expr = Expr::BitXor {
                lhs: Box::new(expr),
                caret_token,
                rhs: Box::new(rhs),
            };
        }
        expr
    })
}

pub fn expr_precedence_bit_and() -> impl Parser<Output = Expr> + Clone {
    let bit_and = {
        ampersand_token()
        .then_optional_whitespace()
        .then(expr_precedence_shift())
    };

    expr_precedence_shift()
    .then(optional_leading_whitespace(bit_and).repeated())
    .map(|(lhs, ops_with_span): (_, WithSpan<_>)| {
        let mut expr = lhs;
        for (ampersand_token, rhs) in ops_with_span.parsed {
            expr = Expr::BitAnd {
                lhs: Box::new(expr),
                ampersand_token,
                rhs: Box::new(rhs),
            };
        }
        expr
    })
}

pub fn expr_precedence_shift() -> impl Parser<Output = Expr> + Clone {
    enum Op {
        Shl {
            shl_token: ShlToken,
            rhs: Expr,
        },
        Shr {
            shr_token: ShrToken,
            rhs: Expr,
        },
    }

    impl Spanned for Op {
        fn span(&self) -> Span {
            match self {
                Op::Shl { shl_token, rhs } => Span::join(shl_token.span(), rhs.span()),
                Op::Shr { shr_token, rhs } => Span::join(shr_token.span(), rhs.span()),
            }
        }
    }

    let op = {
        let shl = {
            shl_token()
            .then_optional_whitespace()
            .then(expr_precedence_add())
            .map(|(shl_token, rhs)| Op::Shl { shl_token, rhs })
        };
        let shr = {
            shr_token()
            .then_optional_whitespace()
            .then(expr_precedence_add())
            .map(|(shr_token, rhs)| Op::Shr { shr_token, rhs })
        };

        shl
        .or(shr)
    };

    expr_precedence_add()
    .then(optional_leading_whitespace(op).repeated())
    .map(|(lhs, ops_with_span): (_, WithSpan<_>)| {
        let mut expr = lhs;
        for op in ops_with_span.parsed {
            expr = match op {
                Op::Shl { shl_token, rhs } => Expr::Shl {
                    lhs: Box::new(expr),
                    shl_token,
                    rhs: Box::new(rhs),
                },
                Op::Shr { shr_token, rhs } => Expr::Shr {
                    lhs: Box::new(expr),
                    shr_token,
                    rhs: Box::new(rhs),
                },
            }
        }
        expr
    })
}

pub fn expr_precedence_add() -> impl Parser<Output = Expr> + Clone {
    enum Op {
        Add {
            add_token: AddToken,
            rhs: Expr,
        },
        Sub {
            sub_token: SubToken,
            rhs: Expr,
        },
    }

    impl Spanned for Op {
        fn span(&self) -> Span {
            match self {
                Op::Add { add_token, rhs } => Span::join(add_token.span(), rhs.span()),
                Op::Sub { sub_token, rhs } => Span::join(sub_token.span(), rhs.span()),
            }
        }
    }

    let op = {
        let add = {
            add_token()
            .then_optional_whitespace()
            .then(expr_precedence_mul())
            .map(|(add_token, rhs)| Op::Add { add_token, rhs })
        };
        let sub = {
            sub_token()
            .then_optional_whitespace()
            .then(expr_precedence_mul())
            .map(|(sub_token, rhs)| Op::Sub { sub_token, rhs })
        };

        add
        .or(sub)
    };

    expr_precedence_mul()
    .then(optional_leading_whitespace(op).repeated())
    .map(|(lhs, ops_with_span): (_, WithSpan<_>)| {
        let mut expr = lhs;
        for op in ops_with_span.parsed {
            expr = match op {
                Op::Add { add_token, rhs } => Expr::Add {
                    lhs: Box::new(expr),
                    add_token,
                    rhs: Box::new(rhs),
                },
                Op::Sub { sub_token, rhs } => Expr::Sub {
                    lhs: Box::new(expr),
                    sub_token,
                    rhs: Box::new(rhs),
                },
            }
        }
        expr
    })
}

pub fn expr_precedence_mul() -> impl Parser<Output = Expr> + Clone {
    enum Op {
        Mul {
            star_token: StarToken,
            rhs: Expr,
        },
        Div {
            forward_slash_token: ForwardSlashToken,
            rhs: Expr,
        },
        Modulo {
            percent_token: PercentToken,
            rhs: Expr,
        },
    }

    impl Spanned for Op {
        fn span(&self) -> Span {
            match self {
                Op::Mul { star_token, rhs } => Span::join(star_token.span(), rhs.span()),
                Op::Div { forward_slash_token, rhs } => Span::join(forward_slash_token.span(), rhs.span()),
                Op::Modulo { percent_token, rhs } => Span::join(percent_token.span(), rhs.span()),
            }
        }
    }

    let op = {
        let mul = {
            star_token()
            .then_optional_whitespace()
            .then(expr_precedence_unary_op())
            .map(|(star_token, rhs)| Op::Mul { star_token, rhs })
        };
        let div = {
            forward_slash_token()
            .then_optional_whitespace()
            .then(expr_precedence_unary_op())
            .map(|(forward_slash_token, rhs)| Op::Div { forward_slash_token, rhs })
        };
        let modulo = {
            percent_token()
            .then_optional_whitespace()
            .then(expr_precedence_unary_op())
            .map(|(percent_token, rhs)| Op::Modulo { percent_token, rhs })
        };

        mul
        .or(div)
        .or(modulo)
    };

    expr_precedence_unary_op()
    .then(optional_leading_whitespace(op).repeated())
    .map(|(lhs, ops_with_span): (_, WithSpan<_>)| {
        let mut expr = lhs;
        for op in ops_with_span.parsed {
            expr = match op {
                Op::Mul { star_token, rhs } => Expr::Mul {
                    lhs: Box::new(expr),
                    star_token,
                    rhs: Box::new(rhs),
                },
                Op::Div { forward_slash_token, rhs } => Expr::Div {
                    lhs: Box::new(expr),
                    forward_slash_token,
                    rhs: Box::new(rhs),
                },
                Op::Modulo { percent_token, rhs } => Expr::Modulo {
                    lhs: Box::new(expr),
                    percent_token,
                    rhs: Box::new(rhs),
                },
            }
        }
        expr
    })
}

pub fn expr_precedence_unary_op() -> impl Parser<Output = Expr> + Clone {
    let not = {
        bang_token()
        .then_optional_whitespace()
        .then(lazy(|| expr()))
        .map(|(bang_token, expr)| {
            Expr::Not {
                bang_token,
                expr: Box::new(expr),
            }
        })
    };

    not
    .or(expr_precedence_projection())
}

pub fn expr_precedence_projection() -> impl Parser<Output = Expr> + Clone {
    enum Projection {
        Index(SquareBrackets<Box<Expr>>),
        MemberOrMethodCall {
            dot_token: DotToken,
            name: Ident,
            method_call_args_opt: Option<Parens<Punctuated<Expr, CommaToken>>>,
        },
    }

    impl Spanned for Projection {
        fn span(&self) -> Span {
            match self {
                Projection::Index(inner) => inner.span(),
                Projection::MemberOrMethodCall { dot_token, name, method_call_args_opt } => {
                    let span_end = match method_call_args_opt {
                        Some(method_call_args) => method_call_args.span(),
                        None => name.span(),
                    };
                    Span::join(dot_token.span(), span_end)
                },
            }
        }
    }

    let projection = {
        let index = {
            square_brackets(padded(lazy(|| expr()).map(Box::new)))
            .map(Projection::Index)
        };
        let member_or_method_call = {
            dot_token()
            .then_optional_whitespace()
            .then(ident())
            .then_optional_whitespace()
            .then(parens(padded(punctuated(lazy(|| expr()), comma_token()))).optional())
            .map(|((dot_token, name), method_call_args_res): ((_, _), Result<_, _>)| {
                Projection::MemberOrMethodCall {
                    dot_token,
                    name,
                    method_call_args_opt: method_call_args_res.ok(),
                }
            })
        };

        member_or_method_call
        .or(index)
    };

    expr_precedence_func_app()
    .then(optional_leading_whitespace(projection).repeated())
    .map(|(expr, projections_with_span): (_, WithSpan<_>)| {
        let mut expr = expr;
        for projection in projections_with_span.parsed {
            match projection {
                Projection::Index(arg) => {
                    expr = Expr::Index {
                        target: Box::new(expr),
                        arg,
                    };
                },
                Projection::MemberOrMethodCall { dot_token, name, method_call_args_opt } => {
                    match method_call_args_opt {
                        Some(args) => {
                            expr = Expr::MethodCall {
                                target: Box::new(expr),
                                dot_token,
                                name,
                                args,
                            };
                        },
                        None => {
                            expr = Expr::FieldProjection {
                                target: Box::new(expr),
                                dot_token,
                                name,
                            };
                        },
                    }
                },
            }
        }
        expr
    })
}

pub fn expr_precedence_func_app() -> impl Parser<Output = Expr> + Clone {
    expr_precedence_atomic()
    .then(
        optional_leading_whitespace(parens(padded(punctuated(lazy(|| expr()), comma_token()))))
        .repeated()
    )
    .map(|(expr, apps_with_span): (_, WithSpan<_>)| {
        let mut expr = expr;
        for args in apps_with_span.parsed {
            expr = Expr::FuncApp {
                func: Box::new(expr),
                args,
            };
        }
        expr
    })
}

pub fn expr_precedence_atomic() -> impl Parser<Output = Expr> + Clone {
    let path = {
        path()
        .map(Expr::Path)
    };
    let string_literal = {
        string_literal()
        .map(Expr::StringLiteral)
    };
    let int_literal = {
        int_literal()
        .map(Expr::IntLiteral)
    };
    let tuple = {
        expr_tuple()
        .map(Expr::Tuple)
    };
    let array = {
        square_brackets(
            optional_leading_whitespace(
                punctuated(lazy(|| expr()), comma_token())
            )
        )
        .map(|elems| Expr::Array { elems })
    };
    let array_repeat = {
        expr_array_repeat()
        .map(Expr::ArrayRepeat)
    };
    let parens = {
        parens(padded(lazy(|| expr())))
        .map(Box::new)
        .map(Expr::Parens)
    };
    let code_block = {
        code_block()
        .map(Expr::CodeBlock)
    };
    
    path
    .or(string_literal)
    .or(int_literal)
    .or(tuple)
    .or(array)
    .or(array_repeat)
    .or(parens)
    .or(code_block)
}
