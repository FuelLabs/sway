pub use crate::priv_prelude::*;

pub enum Expr {
    Path(Path), 
    StringLiteral(StringLiteral),
    IntLiteral(IntLiteral),
    Tuple(ExprTuple),
    CodeBlock(CodeBlock),
    Parens(Box<Parens<Expr>>),
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
            Expr::CodeBlock(code_block) => code_block.span(),
            Expr::Parens(parens) => parens.span(),
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

pub fn expr() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    expr_precedence_comparison().boxed()
}

pub fn expr_precedence_comparison() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
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
    .then(leading_whitespace(op.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
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

pub fn expr_precedence_bit_or() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    let bit_or = {
        pipe_token()
        .then_optional_whitespace()
        .then(expr_precedence_bit_or())
    };

    expr_precedence_bit_xor()
    .then(leading_whitespace(bit_or.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
        None => lhs,
        Some((pipe_token, rhs)) => Expr::BitOr {
            lhs: Box::new(lhs),
            pipe_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_bit_xor() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    let bit_xor = {
        caret_token()
        .then_optional_whitespace()
        .then(expr_precedence_bit_xor())
    };

    expr_precedence_bit_and()
    .then(leading_whitespace(bit_xor.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
        None => lhs,
        Some((caret_token, rhs)) => Expr::BitXor {
            lhs: Box::new(lhs),
            caret_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_bit_and() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    let bit_and = {
        ampersand_token()
        .then_optional_whitespace()
        .then(expr_precedence_bit_and())
    };

    expr_precedence_shift()
    .then(leading_whitespace(bit_and.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
        None => lhs,
        Some((ampersand_token, rhs)) => Expr::BitAnd {
            lhs: Box::new(lhs),
            ampersand_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_shift() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
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

    let op = {
        let shl = {
            shl_token()
            .then_optional_whitespace()
            .then(expr_precedence_shift())
            .map(|(shl_token, rhs)| Op::Shl { shl_token, rhs })
        };
        let shr = {
            shr_token()
            .then_optional_whitespace()
            .then(expr_precedence_shift())
            .map(|(shr_token, rhs)| Op::Shr { shr_token, rhs })
        };

        shl
        .or(shr)
    };

    expr_precedence_add()
    .then(leading_whitespace(op.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
        None => lhs,
        Some(Op::Shl { shl_token, rhs }) => Expr::Shl {
            lhs: Box::new(lhs),
            shl_token,
            rhs: Box::new(rhs),
        },
        Some(Op::Shr { shr_token, rhs }) => Expr::Shr {
            lhs: Box::new(lhs),
            shr_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_add() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
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

    let op = {
        let add = {
            add_token()
            .then_optional_whitespace()
            .then(expr_precedence_add())
            .map(|(add_token, rhs)| Op::Add { add_token, rhs })
        };
        let sub = {
            sub_token()
            .then_optional_whitespace()
            .then(expr_precedence_add())
            .map(|(sub_token, rhs)| Op::Sub { sub_token, rhs })
        };

        add
        .or(sub)
    };

    expr_precedence_mul()
    .then(leading_whitespace(op.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
        None => lhs,
        Some(Op::Add { add_token, rhs }) => Expr::Add {
            lhs: Box::new(lhs),
            add_token,
            rhs: Box::new(rhs),
        },
        Some(Op::Sub { sub_token, rhs }) => Expr::Sub {
            lhs: Box::new(lhs),
            sub_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_mul() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
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

    let op = {
        let mul = {
            star_token()
            .then_optional_whitespace()
            .then(expr_precedence_mul())
            .map(|(star_token, rhs)| Op::Mul { star_token, rhs })
        };
        let div = {
            forward_slash_token()
            .then_optional_whitespace()
            .then(expr_precedence_mul())
            .map(|(forward_slash_token, rhs)| Op::Div { forward_slash_token, rhs })
        };
        let modulo = {
            percent_token()
            .then_optional_whitespace()
            .then(expr_precedence_mul())
            .map(|(percent_token, rhs)| Op::Modulo { percent_token, rhs })
        };

        mul
        .or(div)
        .or(modulo)
    };

    expr_precedence_unary_op()
    .then(leading_whitespace(op.boxed()).or_not())
    .map(|(lhs, op_opt)| match op_opt {
        None => lhs,
        Some(Op::Mul { star_token, rhs }) => Expr::Mul {
            lhs: Box::new(lhs),
            star_token,
            rhs: Box::new(rhs),
        },
        Some(Op::Div { forward_slash_token, rhs }) => Expr::Div {
            lhs: Box::new(lhs),
            forward_slash_token,
            rhs: Box::new(rhs),
        },
        Some(Op::Modulo { percent_token, rhs }) => Expr::Modulo {
            lhs: Box::new(lhs),
            percent_token,
            rhs: Box::new(rhs),
        },
    })
}

pub fn expr_precedence_unary_op() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    recursive(|recurse| {
        let not = {
            bang_token()
            .then_optional_whitespace()
            .then(recurse)
            .map(|(bang_token, expr)| {
                Expr::Not {
                    bang_token,
                    expr: Box::new(expr),
                }
            })
        };

        not
        .or(expr_precedence_projection())
    })
}

pub fn expr_precedence_projection() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    enum Projection {
        Index(SquareBrackets<Box<Expr>>),
        MemberOrMethodCall {
            dot_token: DotToken,
            name: Ident,
            method_call_args_opt: Option<Parens<Punctuated<Expr, CommaToken>>>,
        },
    }

    let projection = {
        let index = {
            square_brackets(padded(expr().map(Box::new)))
            .map(Projection::Index)
        };
        let member_or_method_call = {
            dot_token()
            .then_optional_whitespace()
            .then(ident())
            .then_optional_whitespace()
            .then(parens(padded(punctuated(expr(), comma_token()))).or_not())
            .map(|((dot_token, name), method_call_args_opt)| {
                Projection::MemberOrMethodCall { dot_token, name, method_call_args_opt }
            })
        };

        member_or_method_call
        .or(index)
    };

    expr_precedence_func_app()
    .then(leading_whitespace(projection).repeated())
    .map(|(expr, projections)| {
        let mut expr = expr;
        for projection in projections {
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

pub fn expr_precedence_func_app() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
    expr_precedence_atomic()
    .then(
        leading_whitespace(parens(padded(punctuated(expr(), comma_token()))))
        .repeated()
    )
    .map(|(expr, apps)| {
        let mut expr = expr;
        for args in apps {
            expr = Expr::FuncApp {
                func: Box::new(expr),
                args,
            };
        }
        expr
    })
}

pub fn expr_precedence_atomic() -> impl Parser<char, Expr, Error = Cheap<char, Span>> + Clone {
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
        .boxed()
    };
    let code_block = {
        code_block()
        .map(Expr::CodeBlock)
        .boxed()
    };
    let parens = {
        parens(padded(expr()))
        .map(Box::new)
        .map(Expr::Parens)
        .boxed()
    };
    let path = {
        path()
        .map(Expr::Path)
    };
    
    string_literal
    .or(int_literal)
    .or(tuple)
    .or(code_block)
    .or(parens)
    .or(path)
}
