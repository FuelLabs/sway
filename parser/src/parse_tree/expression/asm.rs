use crate::error::*;
use crate::parse_tree::VarName;
use crate::parser::Rule;
use pest::iterators::Pair;
use pest::Span;

pub(crate) struct AsmExpression<'sc> {
    registers: Vec<AsmRegisterDeclaration<'sc>>,
    body: Vec<AsmOp<'sc>>,
    returns: AsmRegister<'sc>,
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
                    // implicit register return
                    todo!()
                }
            }
        }
        dbg!(&iter.next());
        //        let ops = AsmOp::parse_from)air
        todo!()
    }
}

pub(crate) struct AsmOp<'sc> {
    opcode: &'sc str,
    registers: Vec<AsmRegister<'sc>>,
    immediate: Option<u64>,
    span: Span<'sc>,
}

pub(crate) struct AsmRegister<'sc> {
    name: &'sc str,
}

impl<'sc> AsmOp<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<Self> {
        let warnings = Vec::new();
        let errors = Vec::new();
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
                    todo!()
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

pub(crate) struct AsmRegisterDeclaration<'sc> {
    name: &'sc str,
    initializer: Option<VarName<'sc>>,
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
                    VarName::parse_from_pair,
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
