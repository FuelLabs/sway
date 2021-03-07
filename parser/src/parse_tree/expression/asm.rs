use crate::error::*;
use crate::parse_tree::VarName;
use crate::parser::Rule;
use pest::iterators::Pair;

pub(crate) struct AsmExpression<'sc> {
    registers: Vec<AsmRegister<'sc>>,
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
            AsmRegister::parse_from_pair,
            warnings,
            errors,
            asm_registers,
            return err(warnings, errors)
        );
        todo!()
    }
}

pub(crate) struct AsmOp<'sc> {
    opcode: &'sc str,
    registers: Vec<AsmRegister<'sc>>,
    immediate: Option<u64>,
}

pub(crate) struct AsmRegister<'sc> {
    name: &'sc str,
    initializer: Option<VarName<'sc>>,
}

impl<'sc> AsmRegister<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Vec<Self>> {
        let mut iter = pair.into_inner();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut reg_buf: Vec<AsmRegister> = Vec::new();
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
            reg_buf.push(AsmRegister {
                name: reg_name.as_str(),
                initializer,
            })
        }

        ok(reg_buf, warnings, errors)
    }
}
