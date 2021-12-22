use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span::Span;
use crate::{Ident, TypeInfo};
use pest::iterators::Pair;

use super::Expression;
use crate::type_engine::IntegerBits;

#[derive(Debug, Clone)]
pub struct AsmExpression {
    pub(crate) registers: Vec<AsmRegisterDeclaration>,
    pub(crate) body: Vec<AsmOp>,
    pub(crate) returns: Option<(AsmRegister, Span)>,
    pub(crate) return_type: TypeInfo,
    pub(crate) whole_block_span: Span,
}

impl AsmExpression {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let whole_block_span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _asm_keyword = iter.next();
        let asm_registers = iter.next().unwrap();
        let asm_registers = check!(
            AsmRegisterDeclaration::parse_from_pair(asm_registers, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut asm_op_buf = Vec::new();
        let mut implicit_op_return = None;
        let mut implicit_op_type = None;
        for pair in iter {
            match pair.as_rule() {
                Rule::asm_op => {
                    let op = check!(
                        AsmOp::parse_from_pair(pair, config),
                        continue,
                        warnings,
                        errors
                    );
                    asm_op_buf.push(op);
                }
                Rule::asm_register => {
                    implicit_op_return = Some((
                        check!(
                            AsmRegister::parse_from_pair(pair.clone()),
                            continue,
                            warnings,
                            errors
                        ),
                        Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    ));
                }
                Rule::type_name => {
                    implicit_op_type = Some(check!(
                        TypeInfo::parse_from_pair(pair, config),
                        continue,
                        warnings,
                        errors
                    ));
                }
                a => unreachable!("{:?}", a),
            }
        }
        let return_type = implicit_op_type.unwrap_or(if implicit_op_return.is_some() {
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
        } else {
            TypeInfo::Tuple(Vec::new())
        });

        ok(
            AsmExpression {
                registers: asm_registers,
                body: asm_op_buf,
                returns: implicit_op_return,
                return_type,
                whole_block_span,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AsmOp {
    pub(crate) op_name: Ident,
    pub(crate) op_args: Vec<Ident>,
    pub(crate) span: Span,
    pub(crate) immediate: Option<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AsmRegister {
    pub(crate) name: String,
}

impl AsmRegister {
    fn parse_from_pair(pair: Pair<Rule>) -> CompileResult<Self> {
        ok(
            AsmRegister {
                name: pair.as_str().to_string(),
            },
            vec![],
            vec![],
        )
    }
}

impl From<AsmRegister> for String {
    fn from(register: AsmRegister) -> String {
        register.name
    }
}

impl AsmOp {
    fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut iter = pair.into_inner();
        let opcode = check!(
            Ident::parse_from_pair(iter.next().unwrap(), config),
            return err(warnings, errors),
            warnings,
            errors
        );
        errors.append(&mut disallow_opcode(&opcode));
        let mut args = vec![];
        let mut immediate_value = None;
        for pair in iter {
            match pair.as_rule() {
                Rule::asm_register => {
                    args.push(Ident::new(
                        Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    ));
                }
                Rule::asm_immediate => {
                    immediate_value = Some(Ident::new(
                        Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    ));
                }
                _ => unreachable!(),
            }
        }
        ok(
            AsmOp {
                span,
                op_name: opcode,
                op_args: args,
                immediate: immediate_value,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AsmRegisterDeclaration {
    pub(crate) name: Ident,
    pub(crate) initializer: Option<Expression>,
}

impl AsmRegisterDeclaration {
    fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let iter = pair.into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut reg_buf: Vec<AsmRegisterDeclaration> = Vec::new();
        for pair in iter {
            assert_eq!(pair.as_rule(), Rule::asm_register_declaration);
            let mut iter = pair.into_inner();
            let reg_name = check!(
                Ident::parse_from_pair(iter.next().unwrap(), config),
                return err(warnings, errors),
                warnings,
                errors,
            );
            // if there is still anything in the iterator, then it is a variable expression to be
            // assigned to that register
            let initializer = if let Some(pair) = iter.next() {
                Some(check!(
                    Expression::parse_from_pair(pair, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                ))
            } else {
                None
            };
            reg_buf.push(AsmRegisterDeclaration {
                name: reg_name,
                initializer,
            })
        }

        ok(reg_buf, warnings, errors)
    }
}

fn disallow_opcode(op: &Ident) -> Vec<CompileError> {
    let mut errors = vec![];

    match op.as_str().to_lowercase().as_str() {
        "jnei" => {
            errors.push(CompileError::DisallowedJnei {
                span: op.span().clone(),
            });
        }
        "ji" => {
            errors.push(CompileError::DisallowedJi {
                span: op.span().clone(),
            });
        }
        _ => (),
    };
    errors
}
