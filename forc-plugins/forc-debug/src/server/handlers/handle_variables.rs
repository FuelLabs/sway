use crate::{
    names::register_name,
    server::{
        AdapterError, DapServer, HandlerResult, INSTRUCTIONS_VARIABLE_REF, REGISTERS_VARIABLE_REF,
    },
};
use dap::{requests::VariablesArguments, responses::ResponseBody, types::Variable};
use fuel_vm::fuel_asm::{Imm06, Imm12, Imm18, Imm24, Instruction, RawInstruction, RegId};

impl DapServer {
    /// Processes a variables request, returning all variables and their current values.
    pub(crate) fn handle_variables_command(&self, args: &VariablesArguments) -> HandlerResult {
        let result = self.get_variables(args).map(|variables| {
            ResponseBody::Variables(dap::responses::VariablesResponse { variables })
        });
        match result {
            Ok(result) => HandlerResult::ok(result),
            Err(e) => HandlerResult::err_with_exit(e, 1),
        }
    }

    /// Returns the list of [Variable]s for the current execution state.
    pub(crate) fn get_variables(
        &self,
        args: &VariablesArguments,
    ) -> Result<Vec<Variable>, AdapterError> {
        let executor = self
            .state
            .executors
            .first()
            .ok_or(AdapterError::NoActiveTestExecutor)?;

        let register_variables = executor
            .interpreter
            .registers()
            .iter()
            .enumerate()
            .map(|(index, value)| Variable {
                name: register_name(index),
                value: format!("0x{value:X?}"),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        // Slice out current opcode pc-4..pc and then parse using fuel-asm
        // to return the opcode and its arguments.
        let pc = executor.interpreter.registers()[RegId::PC] as usize;
        let instruction_variables = match Instruction::try_from(RawInstruction::from_be_bytes(
            executor.interpreter.memory()[pc..pc + 4]
                .try_into()
                .unwrap(),
        )) {
            Ok(instruction) => vec![
                ("Opcode", Some(format!("{:?}", instruction.opcode()))),
                ("rA", ra(instruction)),
                ("rB", rb(instruction)),
                ("rC", rc(instruction)),
                ("rD", rd(instruction)),
                ("imm", imm(instruction)),
            ]
            .iter()
            .filter_map(|(name, value)| {
                value.as_ref().map(|value| Variable {
                    name: (*name).to_string(),
                    value: value.to_string(),
                    ..Default::default()
                })
            })
            .collect(),
            Err(_) => vec![],
        };

        match args.variables_reference {
            REGISTERS_VARIABLE_REF => Ok(register_variables),
            INSTRUCTIONS_VARIABLE_REF => Ok(instruction_variables),
            _ => Ok(vec![]),
        }
    }
}

fn reg_id_to_string(reg_id: Option<RegId>) -> Option<String> {
    reg_id.map(|reg_id| register_name(reg_id.into()))
}

fn imm06_to_string(value: Imm06) -> Option<String> {
    Some(format!("0x{:X?}", value.to_u8()))
}

fn imm12_to_string(value: Imm12) -> Option<String> {
    Some(format!("0x{:X?}", value.to_u16()))
}

fn imm18_to_string(value: Imm18) -> Option<String> {
    Some(format!("0x{:X?}", value.to_u32()))
}

fn imm24_to_string(value: Imm24) -> Option<String> {
    Some(format!("0x{:X?}", value.to_u32()))
}

fn ra(instruction: Instruction) -> Option<String> {
    reg_id_to_string(match instruction {
        Instruction::ADD(op) => Some(op.ra()),
        Instruction::AND(op) => Some(op.ra()),
        Instruction::DIV(op) => Some(op.ra()),
        Instruction::EQ(op) => Some(op.ra()),
        Instruction::EXP(op) => Some(op.ra()),
        Instruction::GT(op) => Some(op.ra()),
        Instruction::LT(op) => Some(op.ra()),
        Instruction::MLOG(op) => Some(op.ra()),
        Instruction::MROO(op) => Some(op.ra()),
        Instruction::MOD(op) => Some(op.ra()),
        Instruction::MOVE(op) => Some(op.ra()),
        Instruction::MUL(op) => Some(op.ra()),
        Instruction::NOT(op) => Some(op.ra()),
        Instruction::OR(op) => Some(op.ra()),
        Instruction::SLL(op) => Some(op.ra()),
        Instruction::SRL(op) => Some(op.ra()),
        Instruction::SUB(op) => Some(op.ra()),
        Instruction::XOR(op) => Some(op.ra()),
        Instruction::MLDV(op) => Some(op.ra()),
        Instruction::RET(op) => Some(op.ra()),
        Instruction::RETD(op) => Some(op.ra()),
        Instruction::ALOC(op) => Some(op.ra()),
        Instruction::MCL(op) => Some(op.ra()),
        Instruction::MCP(op) => Some(op.ra()),
        Instruction::MEQ(op) => Some(op.ra()),
        Instruction::BHSH(op) => Some(op.ra()),
        Instruction::BHEI(op) => Some(op.ra()),
        Instruction::BURN(op) => Some(op.ra()),
        Instruction::CALL(op) => Some(op.ra()),
        Instruction::CCP(op) => Some(op.ra()),
        Instruction::CROO(op) => Some(op.ra()),
        Instruction::CSIZ(op) => Some(op.ra()),
        Instruction::CB(op) => Some(op.ra()),
        Instruction::LDC(op) => Some(op.ra()),
        Instruction::LOG(op) => Some(op.ra()),
        Instruction::LOGD(op) => Some(op.ra()),
        Instruction::MINT(op) => Some(op.ra()),
        Instruction::RVRT(op) => Some(op.ra()),
        Instruction::SCWQ(op) => Some(op.ra()),
        Instruction::SRW(op) => Some(op.ra()),
        Instruction::SRWQ(op) => Some(op.ra()),
        Instruction::SWW(op) => Some(op.ra()),
        Instruction::SWWQ(op) => Some(op.ra()),
        Instruction::TR(op) => Some(op.ra()),
        Instruction::TRO(op) => Some(op.ra()),
        Instruction::ECK1(op) => Some(op.ra()),
        Instruction::ECR1(op) => Some(op.ra()),
        Instruction::ED19(op) => Some(op.ra()),
        Instruction::K256(op) => Some(op.ra()),
        Instruction::S256(op) => Some(op.ra()),
        Instruction::TIME(op) => Some(op.ra()),
        Instruction::FLAG(op) => Some(op.ra()),
        Instruction::BAL(op) => Some(op.ra()),
        Instruction::JMP(op) => Some(op.ra()),
        Instruction::JNE(op) => Some(op.ra()),
        Instruction::SMO(op) => Some(op.ra()),
        Instruction::ADDI(op) => Some(op.ra()),
        Instruction::ANDI(op) => Some(op.ra()),
        Instruction::DIVI(op) => Some(op.ra()),
        Instruction::EXPI(op) => Some(op.ra()),
        Instruction::MODI(op) => Some(op.ra()),
        Instruction::MULI(op) => Some(op.ra()),
        Instruction::ORI(op) => Some(op.ra()),
        Instruction::SLLI(op) => Some(op.ra()),
        Instruction::SRLI(op) => Some(op.ra()),
        Instruction::SUBI(op) => Some(op.ra()),
        Instruction::XORI(op) => Some(op.ra()),
        Instruction::JNEI(op) => Some(op.ra()),
        Instruction::LB(op) => Some(op.ra()),
        Instruction::LW(op) => Some(op.ra()),
        Instruction::SB(op) => Some(op.ra()),
        Instruction::SW(op) => Some(op.ra()),
        Instruction::MCPI(op) => Some(op.ra()),
        Instruction::GTF(op) => Some(op.ra()),
        Instruction::MCLI(op) => Some(op.ra()),
        Instruction::GM(op) => Some(op.ra()),
        Instruction::MOVI(op) => Some(op.ra()),
        Instruction::JNZI(op) => Some(op.ra()),
        Instruction::JMPF(op) => Some(op.ra()),
        Instruction::JMPB(op) => Some(op.ra()),
        Instruction::JNZF(op) => Some(op.ra()),
        Instruction::JNZB(op) => Some(op.ra()),
        Instruction::JNEF(op) => Some(op.ra()),
        Instruction::JNEB(op) => Some(op.ra()),
        Instruction::CFE(op) => Some(op.ra()),
        Instruction::CFS(op) => Some(op.ra()),
        Instruction::WDCM(op) => Some(op.ra()),
        Instruction::WQCM(op) => Some(op.ra()),
        Instruction::WDOP(op) => Some(op.ra()),
        Instruction::WQOP(op) => Some(op.ra()),
        Instruction::WDML(op) => Some(op.ra()),
        Instruction::WQML(op) => Some(op.ra()),
        Instruction::WDDV(op) => Some(op.ra()),
        Instruction::WQDV(op) => Some(op.ra()),
        Instruction::WDMD(op) => Some(op.ra()),
        Instruction::WQMD(op) => Some(op.ra()),
        Instruction::WDAM(op) => Some(op.ra()),
        Instruction::WQAM(op) => Some(op.ra()),
        Instruction::WDMM(op) => Some(op.ra()),
        Instruction::WQMM(op) => Some(op.ra()),
        Instruction::ECAL(op) => Some(op.ra()),
        _ => None,
    })
}

fn rb(instruction: Instruction) -> Option<String> {
    reg_id_to_string(match instruction {
        Instruction::ADD(op) => Some(op.rb()),
        Instruction::AND(op) => Some(op.rb()),
        Instruction::DIV(op) => Some(op.rb()),
        Instruction::EQ(op) => Some(op.rb()),
        Instruction::EXP(op) => Some(op.rb()),
        Instruction::GT(op) => Some(op.rb()),
        Instruction::LT(op) => Some(op.rb()),
        Instruction::MLOG(op) => Some(op.rb()),
        Instruction::MROO(op) => Some(op.rb()),
        Instruction::MOD(op) => Some(op.rb()),
        Instruction::MOVE(op) => Some(op.rb()),
        Instruction::MUL(op) => Some(op.rb()),
        Instruction::NOT(op) => Some(op.rb()),
        Instruction::OR(op) => Some(op.rb()),
        Instruction::SLL(op) => Some(op.rb()),
        Instruction::SRL(op) => Some(op.rb()),
        Instruction::SUB(op) => Some(op.rb()),
        Instruction::XOR(op) => Some(op.rb()),
        Instruction::MLDV(op) => Some(op.rb()),
        Instruction::RETD(op) => Some(op.rb()),
        Instruction::MCL(op) => Some(op.rb()),
        Instruction::MCP(op) => Some(op.rb()),
        Instruction::MEQ(op) => Some(op.rb()),
        Instruction::BHSH(op) => Some(op.rb()),
        Instruction::BURN(op) => Some(op.rb()),
        Instruction::CALL(op) => Some(op.rb()),
        Instruction::CCP(op) => Some(op.rb()),
        Instruction::CROO(op) => Some(op.rb()),
        Instruction::CSIZ(op) => Some(op.rb()),
        Instruction::LDC(op) => Some(op.rb()),
        Instruction::LOG(op) => Some(op.rb()),
        Instruction::LOGD(op) => Some(op.rb()),
        Instruction::MINT(op) => Some(op.rb()),
        Instruction::SCWQ(op) => Some(op.rb()),
        Instruction::SRW(op) => Some(op.rb()),
        Instruction::SRWQ(op) => Some(op.rb()),
        Instruction::SWW(op) => Some(op.rb()),
        Instruction::SWWQ(op) => Some(op.rb()),
        Instruction::TR(op) => Some(op.rb()),
        Instruction::TRO(op) => Some(op.rb()),
        Instruction::ECK1(op) => Some(op.rb()),
        Instruction::ECR1(op) => Some(op.rb()),
        Instruction::ED19(op) => Some(op.rb()),
        Instruction::K256(op) => Some(op.rb()),
        Instruction::S256(op) => Some(op.rb()),
        Instruction::TIME(op) => Some(op.rb()),
        Instruction::BAL(op) => Some(op.rb()),
        Instruction::JNE(op) => Some(op.rb()),
        Instruction::SMO(op) => Some(op.rb()),
        Instruction::ADDI(op) => Some(op.rb()),
        Instruction::ANDI(op) => Some(op.rb()),
        Instruction::DIVI(op) => Some(op.rb()),
        Instruction::EXPI(op) => Some(op.rb()),
        Instruction::MODI(op) => Some(op.rb()),
        Instruction::MULI(op) => Some(op.rb()),
        Instruction::ORI(op) => Some(op.rb()),
        Instruction::SLLI(op) => Some(op.rb()),
        Instruction::SRLI(op) => Some(op.rb()),
        Instruction::SUBI(op) => Some(op.rb()),
        Instruction::XORI(op) => Some(op.rb()),
        Instruction::JNEI(op) => Some(op.rb()),
        Instruction::LB(op) => Some(op.rb()),
        Instruction::LW(op) => Some(op.rb()),
        Instruction::SB(op) => Some(op.rb()),
        Instruction::SW(op) => Some(op.rb()),
        Instruction::MCPI(op) => Some(op.rb()),
        Instruction::GTF(op) => Some(op.rb()),
        Instruction::JNZF(op) => Some(op.rb()),
        Instruction::JNZB(op) => Some(op.rb()),
        Instruction::JNEF(op) => Some(op.rb()),
        Instruction::JNEB(op) => Some(op.rb()),
        Instruction::WDCM(op) => Some(op.rb()),
        Instruction::WQCM(op) => Some(op.rb()),
        Instruction::WDOP(op) => Some(op.rb()),
        Instruction::WQOP(op) => Some(op.rb()),
        Instruction::WDML(op) => Some(op.rb()),
        Instruction::WQML(op) => Some(op.rb()),
        Instruction::WDDV(op) => Some(op.rb()),
        Instruction::WQDV(op) => Some(op.rb()),
        Instruction::WDMD(op) => Some(op.rb()),
        Instruction::WQMD(op) => Some(op.rb()),
        Instruction::WDAM(op) => Some(op.rb()),
        Instruction::WQAM(op) => Some(op.rb()),
        Instruction::WDMM(op) => Some(op.rb()),
        Instruction::WQMM(op) => Some(op.rb()),
        Instruction::ECAL(op) => Some(op.rb()),
        _ => None,
    })
}

fn rc(instruction: Instruction) -> Option<String> {
    reg_id_to_string(match instruction {
        Instruction::ADD(op) => Some(op.rc()),
        Instruction::AND(op) => Some(op.rc()),
        Instruction::DIV(op) => Some(op.rc()),
        Instruction::EQ(op) => Some(op.rc()),
        Instruction::EXP(op) => Some(op.rc()),
        Instruction::GT(op) => Some(op.rc()),
        Instruction::LT(op) => Some(op.rc()),
        Instruction::MLOG(op) => Some(op.rc()),
        Instruction::MROO(op) => Some(op.rc()),
        Instruction::MOD(op) => Some(op.rc()),
        Instruction::MUL(op) => Some(op.rc()),
        Instruction::OR(op) => Some(op.rc()),
        Instruction::SLL(op) => Some(op.rc()),
        Instruction::SRL(op) => Some(op.rc()),
        Instruction::SUB(op) => Some(op.rc()),
        Instruction::XOR(op) => Some(op.rc()),
        Instruction::MLDV(op) => Some(op.rc()),
        Instruction::MCP(op) => Some(op.rc()),
        Instruction::MEQ(op) => Some(op.rc()),
        Instruction::CALL(op) => Some(op.rc()),
        Instruction::CCP(op) => Some(op.rc()),
        Instruction::LDC(op) => Some(op.rc()),
        Instruction::LOG(op) => Some(op.rc()),
        Instruction::LOGD(op) => Some(op.rc()),
        Instruction::SCWQ(op) => Some(op.rc()),
        Instruction::SRW(op) => Some(op.rc()),
        Instruction::SRWQ(op) => Some(op.rc()),
        Instruction::SWW(op) => Some(op.rc()),
        Instruction::SWWQ(op) => Some(op.rc()),
        Instruction::TR(op) => Some(op.rc()),
        Instruction::TRO(op) => Some(op.rc()),
        Instruction::ECK1(op) => Some(op.rc()),
        Instruction::ECR1(op) => Some(op.rc()),
        Instruction::ED19(op) => Some(op.rc()),
        Instruction::K256(op) => Some(op.rc()),
        Instruction::S256(op) => Some(op.rc()),
        Instruction::BAL(op) => Some(op.rc()),
        Instruction::JNE(op) => Some(op.rc()),
        Instruction::SMO(op) => Some(op.rc()),
        Instruction::JNEF(op) => Some(op.rc()),
        Instruction::JNEB(op) => Some(op.rc()),
        Instruction::WDCM(op) => Some(op.rc()),
        Instruction::WQCM(op) => Some(op.rc()),
        Instruction::WDOP(op) => Some(op.rc()),
        Instruction::WQOP(op) => Some(op.rc()),
        Instruction::WDML(op) => Some(op.rc()),
        Instruction::WQML(op) => Some(op.rc()),
        Instruction::WDDV(op) => Some(op.rc()),
        Instruction::WQDV(op) => Some(op.rc()),
        Instruction::WDMD(op) => Some(op.rc()),
        Instruction::WQMD(op) => Some(op.rc()),
        Instruction::WDAM(op) => Some(op.rc()),
        Instruction::WQAM(op) => Some(op.rc()),
        Instruction::WDMM(op) => Some(op.rc()),
        Instruction::WQMM(op) => Some(op.rc()),
        Instruction::ECAL(op) => Some(op.rc()),
        _ => None,
    })
}

fn rd(instruction: Instruction) -> Option<String> {
    reg_id_to_string(match instruction {
        Instruction::MLDV(op) => Some(op.rd()),
        Instruction::MEQ(op) => Some(op.rd()),
        Instruction::CALL(op) => Some(op.rd()),
        Instruction::CCP(op) => Some(op.rd()),
        Instruction::LOG(op) => Some(op.rd()),
        Instruction::LOGD(op) => Some(op.rd()),
        Instruction::SRWQ(op) => Some(op.rd()),
        Instruction::SWWQ(op) => Some(op.rd()),
        Instruction::TRO(op) => Some(op.rd()),
        Instruction::WDMD(op) => Some(op.rd()),
        Instruction::WQMD(op) => Some(op.rd()),
        Instruction::WDAM(op) => Some(op.rd()),
        Instruction::WQAM(op) => Some(op.rd()),
        Instruction::WDMM(op) => Some(op.rd()),
        Instruction::WQMM(op) => Some(op.rd()),
        Instruction::ECAL(op) => Some(op.rd()),
        _ => None,
    })
}

fn imm(instruction: Instruction) -> Option<String> {
    match instruction {
        Instruction::ADDI(op) => imm12_to_string(op.imm12()),
        Instruction::ANDI(op) => imm12_to_string(op.imm12()),
        Instruction::DIVI(op) => imm12_to_string(op.imm12()),
        Instruction::EXPI(op) => imm12_to_string(op.imm12()),
        Instruction::MODI(op) => imm12_to_string(op.imm12()),
        Instruction::MULI(op) => imm12_to_string(op.imm12()),
        Instruction::ORI(op) => imm12_to_string(op.imm12()),
        Instruction::SLLI(op) => imm12_to_string(op.imm12()),
        Instruction::SRLI(op) => imm12_to_string(op.imm12()),
        Instruction::SUBI(op) => imm12_to_string(op.imm12()),
        Instruction::XORI(op) => imm12_to_string(op.imm12()),
        Instruction::JNEI(op) => imm12_to_string(op.imm12()),
        Instruction::LB(op) => imm12_to_string(op.imm12()),
        Instruction::LW(op) => imm12_to_string(op.imm12()),
        Instruction::SB(op) => imm12_to_string(op.imm12()),
        Instruction::SW(op) => imm12_to_string(op.imm12()),
        Instruction::MCPI(op) => imm12_to_string(op.imm12()),
        Instruction::GTF(op) => imm12_to_string(op.imm12()),
        Instruction::MCLI(op) => imm18_to_string(op.imm18()),
        Instruction::GM(op) => imm18_to_string(op.imm18()),
        Instruction::MOVI(op) => imm18_to_string(op.imm18()),
        Instruction::JNZI(op) => imm18_to_string(op.imm18()),
        Instruction::JMPF(op) => imm18_to_string(op.imm18()),
        Instruction::JMPB(op) => imm18_to_string(op.imm18()),
        Instruction::JNZF(op) => imm12_to_string(op.imm12()),
        Instruction::JNZB(op) => imm12_to_string(op.imm12()),
        Instruction::JNEF(op) => imm06_to_string(op.imm06()),
        Instruction::JNEB(op) => imm06_to_string(op.imm06()),
        Instruction::JI(op) => imm24_to_string(op.imm24()),
        Instruction::CFEI(op) => imm24_to_string(op.imm24()),
        Instruction::CFSI(op) => imm24_to_string(op.imm24()),
        Instruction::PSHL(op) => imm24_to_string(op.imm24()),
        Instruction::PSHH(op) => imm24_to_string(op.imm24()),
        Instruction::POPL(op) => imm24_to_string(op.imm24()),
        Instruction::POPH(op) => imm24_to_string(op.imm24()),
        Instruction::WDCM(op) => imm06_to_string(op.imm06()),
        Instruction::WQCM(op) => imm06_to_string(op.imm06()),
        Instruction::WDOP(op) => imm06_to_string(op.imm06()),
        Instruction::WQOP(op) => imm06_to_string(op.imm06()),
        Instruction::WDML(op) => imm06_to_string(op.imm06()),
        Instruction::WQML(op) => imm06_to_string(op.imm06()),
        Instruction::WDDV(op) => imm06_to_string(op.imm06()),
        Instruction::WQDV(op) => imm06_to_string(op.imm06()),
        Instruction::SRW(op) => imm06_to_string(op.imm06()),
        _ => None,
    }
}
