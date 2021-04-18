use crate::error::*;
use crate::parser::Rule;
use crate::vendored_vm::Opcode;
use crate::Ident;
use pest::iterators::Pair;
use pest::Span;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(crate) struct AsmExpression<'sc> {
    pub(crate) registers: Vec<AsmRegisterDeclaration<'sc>>,
    pub(crate) body: Vec<AsmOp<'sc>>,
    pub(crate) unique_registers: HashSet<AsmRegister>,
    pub(crate) returns: Option<AsmRegister>,
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

        let unique_registers = asm_op_buf
            .iter()
            .map(|x| x.op.get_register_names())
            .flatten()
            .collect::<HashSet<_>>();
        ok(
            AsmExpression {
                registers: asm_registers,
                body: asm_op_buf,
                returns: implicit_op_return,
                unique_registers,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AsmOp<'sc> {
    op: Opcode,
    span: Span<'sc>,
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

impl<'sc> AsmOp<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let warnings = Vec::new();
        let errors = Vec::new();
        let span = pair.as_span();
        let mut iter = pair.into_inner();
        let opcode = iter.next().unwrap().as_str();
        //        let mut registers_buf = Vec::new();
        //        let mut immediate: Option<u64> = None;
        let mut args = vec![];
        while let Some(pair) = iter.next() {
            match pair.as_rule() {
                Rule::asm_register | Rule::asm_immediate => {
                    args.push(pair.as_str());
                }
                _ => unreachable!(),
            }
        }
        let op = Opcode::parse(opcode, &args).expect("handle this err");
        ok(AsmOp { span, op }, warnings, errors)
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
