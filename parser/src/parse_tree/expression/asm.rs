use crate::error::*;
use crate::parse_tree::Ident;
use crate::parser::Rule;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub(crate) struct AsmExpression<'sc> {
    pub(crate) registers: Vec<AsmRegisterDeclaration<'sc>>,
    pub(crate) body: Vec<AsmOp<'sc>>,
    pub(crate) returns: Option<AsmRegister<'sc>>,
}

impl<'sc> AsmExpression<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _asm_keyword = iter.next();
        let asm_registers = iter.next().unwrap();
        let asm_registers = eval!(
            AsmRegisterDeclaration::parse_from_pair,
            warnings,
            errors,
            asm_registers,
            return err(warnings, errors)
        );
        let mut asm_op_buf = Vec::new();
        let mut implicit_op_return: Option<AsmRegister> = None;
        while let Some(pair) = iter.next() {
            match pair.as_rule() {
                Rule::asm_op => {
                    let op = eval!(AsmOp::parse_from_pair, warnings, errors, pair, continue);
                    asm_op_buf.push(op);
                }
                Rule::asm_register => {
                    implicit_op_return = Some(eval!(
                        AsmRegister::parse_from_pair,
                        warnings,
                        errors,
                        pair,
                        continue
                    ));
                }
                a => unreachable!("{:?}", a),
            }
        }
        ok(
            AsmExpression {
                registers: asm_registers,
                body: asm_op_buf,
                returns: implicit_op_return,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AsmOp<'sc> {
    opcode: &'sc str,
    registers: Vec<AsmRegister<'sc>>,
    immediate: Option<u64>,
    span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub(crate) struct AsmRegister<'sc> {
    name: &'sc str,
}

impl<'sc> AsmRegister<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        ok(
            AsmRegister {
                name: pair.as_str(),
            },
            vec![],
            vec![],
        )
    }
}

impl<'sc> AsmOp<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let warnings = Vec::new();
        let mut errors = Vec::new();
        let span = pair.as_span();
        let mut iter = pair.into_inner();
        // TODO map to the actual enum from the VM
        let opcode = iter.next().unwrap().as_str();
        let mut registers_buf = Vec::new();
        let mut immediate: Option<u64> = None;
        while let Some(pair) = iter.next() {
            match pair.as_rule() {
                Rule::asm_register => {
                    registers_buf.push(AsmRegister {
                        name: pair.as_str(),
                    });
                }
                Rule::asm_immediate => {
                    let span = pair.as_span();
                    let num = pair.into_inner().next().unwrap();
                    if immediate.is_some() {
                        errors.push(CompileError::MultipleImmediates(span.clone()));
                    }
                    immediate = Some(match num.into_inner().next().unwrap().as_str().parse() {
                        Ok(o) => o,
                        Err(_) => {
                            errors.push(CompileError::Internal(
                                "Attempted to parse u64 from invalid number",
                                span,
                            ));
                            0
                        }
                    });
                }
                _ => unreachable!(),
            }
        }
        ok(
            AsmOp {
                span,
                opcode,
                registers: registers_buf,
                immediate,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AsmRegisterDeclaration<'sc> {
    name: &'sc str,
    initializer: Option<Ident<'sc>>,
}

impl<'sc> AsmRegisterDeclaration<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Vec<Self>> {
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
                    Ident::parse_from_pair,
                    warnings,
                    errors,
                    pair,
                    return err(warnings, errors)
                ))
            } else {
                None
            };
            reg_buf.push(AsmRegisterDeclaration {
                name: reg_name.as_str(),
                initializer,
            })
        }

        ok(reg_buf, warnings, errors)
    }
}
