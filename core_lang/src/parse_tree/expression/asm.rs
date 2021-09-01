use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span::Span;
use crate::{Ident, TypeInfo};
use pest::iterators::Pair;

use super::Expression;
use crate::types::IntegerBits;

#[derive(Debug, Clone)]
pub struct AsmExpression<'sc> {
    pub(crate) registers: Vec<AsmRegisterDeclaration<'sc>>,
    pub(crate) body: Vec<AsmOp<'sc>>,
    pub(crate) returns: Option<(AsmRegister, Span<'sc>)>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) whole_block_span: Span<'sc>,
}

impl<'sc> AsmExpression<'sc> {
    pub(crate) fn parse_from_pair(
        input: (Pair<'sc, Rule>, Option<BuildConfig>),
    ) -> CompileResult<'sc, Self> {
        let (pair, config) = input;
        let path = config.map(|config| config.dir_of_code);
        let whole_block_span = Span {
            span: pair.as_span(),
            path,
        };
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _asm_keyword = iter.next();
        let asm_registers = iter.next().unwrap();
        let asm_registers = eval!(
            AsmRegisterDeclaration::parse_from_pair,
            warnings,
            errors,
            (asm_registers, config),
            return err(warnings, errors)
        );
        let mut asm_op_buf = Vec::new();
        let mut implicit_op_return = None;
        let mut implicit_op_type = None;
        while let Some(pair) = iter.next() {
            match pair.as_rule() {
                Rule::asm_op => {
                    let op = eval!(AsmOp::parse_from_pair, warnings, errors, (pair, config), continue);
                    asm_op_buf.push(op);
                }
                Rule::asm_register => {
                    implicit_op_return = Some((
                        eval!(
                            AsmRegister::parse_from_pair,
                            warnings,
                            errors,
                            (pair, config),
                            continue
                        ),
                        Span {
                            span: pair.as_span(),
                            path,
                        },
                    ));
                }
                Rule::type_name => {
                    implicit_op_type = Some(eval!(
                        TypeInfo::parse_from_pair,
                        warnings,
                        errors,
                        (pair, config),
                        continue
                    ));
                }
                a => unreachable!("{:?}", a),
            }
        }
        let return_type = implicit_op_type.unwrap_or(if implicit_op_return.is_some() {
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
        } else {
            TypeInfo::Unit
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
pub(crate) struct AsmOp<'sc> {
    pub(crate) op_name: Ident<'sc>,
    pub(crate) op_args: Vec<Ident<'sc>>,
    pub(crate) span: Span<'sc>,
    pub(crate) immediate: Option<Ident<'sc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AsmRegister {
    pub(crate) name: String,
}

impl<'sc> AsmRegister {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        ok(
            AsmRegister {
                name: pair.as_str().to_string(),
            },
            vec![],
            vec![],
        )
    }
}

impl Into<String> for AsmRegister {
    fn into(self) -> String {
        self.name.clone()
    }
}

impl<'sc> AsmOp<'sc> {
    fn parse_from_pair(input: (Pair<'sc, Rule>, Option<BuildConfig>)) -> CompileResult<'sc, Self> {
        let (pair, config) = input;
        let path = config.map(|config| config.dir_of_code);
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = Span {
            span: pair.as_span(),
            path,
        };
        let mut iter = pair.into_inner();
        let opcode = eval!(
            Ident::parse_from_pair,
            warnings,
            errors,
            (iter.next().unwrap(), config),
            return err(warnings, errors)
        );
        let mut args = vec![];
        let mut immediate_value = None;
        while let Some(pair) = iter.next() {
            match pair.as_rule() {
                Rule::asm_register => {
                    args.push(Ident {
                        primary_name: pair.as_str(),
                        span: Span {
                            span: pair.as_span(),
                            path,
                        },
                    });
                }
                Rule::asm_immediate => {
                    immediate_value = Some(Ident {
                        primary_name: pair.as_str().trim(),
                        span: Span {
                            span: pair.as_span(),
                            path,
                        },
                    });
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
pub(crate) struct AsmRegisterDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) initializer: Option<Expression<'sc>>,
    pub(crate) name_span: Span<'sc>,
}

impl<'sc> AsmRegisterDeclaration<'sc> {
    fn parse_from_pair(
        input: (Pair<'sc, Rule>, Option<BuildConfig>),
    ) -> CompileResult<'sc, Vec<Self>> {
        let (pair, config) = input;
        let mut iter = pair.into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut reg_buf: Vec<AsmRegisterDeclaration> = Vec::new();
        while let Some(pair) = iter.next() {
            assert_eq!(pair.as_rule(), Rule::asm_register_declaration);
            let mut iter = pair.into_inner();
            let reg_name = iter.next().unwrap();
            // if there is still anything in the iterator, then it is a variable expression to be
            // assigned to that register
            let initializer = if let Some(pair) = iter.next() {
                Some(eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    (pair, config),
                    return err(warnings, errors)
                ))
            } else {
                None
            };
            reg_buf.push(AsmRegisterDeclaration {
                name: reg_name.as_str(),
                name_span: Span {
                    span: reg_name.as_span(),
                    path: config.map(|config| config.dir_of_code),
                },
                initializer,
            })
        }

        ok(reg_buf, warnings, errors)
    }
}
