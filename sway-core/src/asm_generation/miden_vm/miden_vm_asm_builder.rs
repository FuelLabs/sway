#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};
mod miden_op;
pub use miden_op::MidenAsmOp;

use crate::{
    asm_generation::{
        asm_builder::{AsmBuilder, AsmBuilderResult},
        from_ir::StateAccessType,
        miden_vm::miden_vm_asm_builder::miden_op::{MidenStackValue, Push},
        ProgramKind,
    },
    asm_lang::Label,
    error::*,
    metadata::MetadataManager,
};

use sway_error::error::CompileError;
use sway_ir::{Context, *};
use sway_types::Span;

pub use miden_op::DirectOp;

#[derive(Default)]
pub struct MidenVMAsmSection {
    ops: Vec<miden_core::Operation>,
}

/// A procedure block is used to define a frequently-used sequence of instructions. A procedure must
/// start with a proc.<label>.<number of locals> instruction and terminate with an end instruction.
/// [ref](https://wiki.polygon.technology/docs/miden/user_docs/assembly/code_organization#procedures)
pub struct Procedure {
    number_of_locals: u32,
    ops: Vec<MidenAsmOp>,
}

#[derive(Default, Copy, Clone)]
pub struct StackLabel(usize);

#[derive(Default)]
/// an abstract stack, used to organize instruction code
pub struct StackManager {
    idx: StackLabel,
}

impl StackManager {
    pub fn insert_value(&mut self) -> StackLabel {
        let label = self.idx;
        self.idx.0 += 1;
        label
    }
    pub fn size(&self) -> usize {
        self.idx.0
    }
}
// will likely want to minimize these names eventually
// for now, we can use function names for readability
pub type ProcedureName = String;

pub type ProcedureMap = HashMap<ProcedureName, Procedure>;

/// MidenVM Asm is built in the following way:
/// Function bodies are abstracted into [Procedures]
/// Arguments to functions are evaluated before the exec.proc is issued
pub struct MidenVMAsmBuilder<'ir> {
    procedure_map: ProcedureMap,
    buf: Vec<MidenAsmOp>,
    #[allow(dead_code)]
    program_kind: ProgramKind,
    pub(super) func_label_map: HashMap<Function, (Label, Label)>,
    /// If we are currently compiling a procedure, then ops get inserted to that procedure
    /// otherwise, we push to the main procedure
    active_procedure: Option<ProcedureName>,

    // IR context we're compiling.
    context: &'ir Context,

    // Metadata manager for converting metadata to Spans, etc.
    md_mgr: MetadataManager,

    // Monotonically increasing unique identifier for label generation.
    label_idx: usize,

    // stack label generator
    stack_manager: StackManager,
}

impl std::fmt::Debug for MidenVMAsmBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ops = self
            .buf
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join("\n");
        writeln!(f, "Stack size: {}", self.stack_manager.size())?;
        write!(f, "{ops}")
    }
}

impl MidenVMAsmSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(&self) -> usize {
        todo!()
    }
}

pub struct MidenVMAsmBuilderResult {
    pub ops: Vec<miden_op::DirectOp>,
}

pub type MidenVMAbiResult = ();

impl<'ir> AsmBuilder for MidenVMAsmBuilder<'ir> {
    fn compile_function(&mut self, function: Function) -> CompileResult<()> {
        self.compile_function(function)
    }

    fn finalize(&self) -> AsmBuilderResult {
        self.finalize_inner()
    }

    fn func_to_labels(&mut self, func: &Function) -> (Label, Label) {
        self.func_label_map.get(func).cloned().unwrap_or_else(|| {
            let labels = (self.get_label(), self.get_label());
            self.func_label_map.insert(*func, labels);
            labels
        })
    }
}

#[allow(unused_variables)]
#[allow(dead_code)]
impl<'ir> MidenVMAsmBuilder<'ir> {
    pub fn new(program_kind: ProgramKind, context: &'ir Context) -> Self {
        MidenVMAsmBuilder {
            procedure_map: Default::default(),
            buf: Default::default(),
            func_label_map: Default::default(),
            program_kind,
            context,
            md_mgr: MetadataManager::default(),
            label_idx: 0,
            stack_manager: Default::default(),
            active_procedure: None,
        }
    }

    #[allow(unreachable_code)]
    pub fn finalize(&self) -> AsmBuilderResult {
        AsmBuilderResult::MidenVM(MidenVMAsmBuilderResult { ops: todo!() })
    }

    fn generate_constructor(
        &self,
        is_payable: bool,
        data_size: u32,
        data_offset: u32,
    ) -> MidenVMAsmSection {
        todo!()
    }

    fn copy_contract_code_to_memory(
        &self,
        s: &mut MidenVMAsmSection,
        data_size: u32,
        data_offset: u32,
    ) {
        todo!()
    }

    fn generate_function(&mut self) -> MidenVMAsmSection {
        todo!()
    }

    fn finalize_procedure_map(&self) -> Vec<DirectOp> {
        let mut buf: Vec<DirectOp> = Vec::with_capacity(self.procedure_map.len());
        for (ref proc_name, proc) in &self.procedure_map {
            buf.push(DirectOp::procedure_decl(
                proc_name.to_string(),
                proc.number_of_locals,
            ));
            for op in &proc.ops {
                buf.append(&mut self.render_op(op))
            }
        }
        buf
    }

    pub fn finalize_inner(&self) -> AsmBuilderResult {
        // take each procedure and serialize it with the format described [here](https://wiki.polygon.technology/docs/miden/user_docs/assembly/code_organization/#procedures)
        let mut program = self.finalize_procedure_map();
        // main fn is a call into one of the procedure_map fns
        program.push(DirectOp::begin());
        for op in &self.buf {
            program.append(&mut self.render_op(op))
        }
        program.push(DirectOp::end());

        AsmBuilderResult::MidenVM(MidenVMAsmBuilderResult { ops: program })
    }

    fn empty_span() -> Span {
        let msg = "unknown source location";
        Span::new(Arc::from(msg), 0, msg.len(), None).unwrap()
    }

    fn get_label(&mut self) -> Label {
        let next_val = self.label_idx;
        self.label_idx += 1;
        Label(self.label_idx)
    }

    /// compiles some value and ensures it is on the top of the stack
    fn push_value_to_stack(&mut self, value: &Value) -> CompileResult<()> {
        todo!()
    }

    /// ref: https://wiki.polygon.technology/docs/miden/user_docs/assembly/flow_control#conditional-execution
    /// TODO:
    /// Potentially use conditional dropping if the cost of the branches is less than the cost to execute the if
    /// op

    fn compile_conditional_branch(
        &mut self,
        cond_value: &Value,
        true_block: &BranchToWithArgs,
        false_block: &BranchToWithArgs,
    ) {
        // compile the condition, make sure it is at the top of the stack
        // generate the `if.true`
        // generate the body
        // generate the `else`
        // generate the else block
        self.push_value_to_stack(cond_value);
        self.compile_branch(true_block);
        self.compile_branch(true_block);
        // todo need to figure out how to handle the compile results here

        /*
        self.buf.push(MidenAsmOp::If {
            condition,
            true_branch,
            else_branch,
        });
        */
        todo!()
    }

    fn compile_branch(&mut self, to_block: &BranchToWithArgs) {
        todo!()
        // self.compile_branch_to_phi_value(to_block);

        // let label = self.block_to_label(&to_block.block);
        // self.cur_bytecode.push(Op::jump_to_label(label));
    }
    pub(super) fn compile_call(&mut self, instr_val: &Value, function: &Function, args: &[Value]) {
        // Assume the arguments have been compiled
        // We standardize on the arguments being the top N values in the stack
        // This compilation task simply has to run the procedure

        // The `args` are already on the top of the stack, in the correct order. Load them into local memory
        // let mut arg_mapping = Vec::new();

        todo!(
            "Unable to compile fn call {}, need to design function calls.",
            function.get_name(self.context)
        )
    }
    pub(super) fn compile_instruction(&mut self, instr_val: &Value) {
        if let Some(instruction) = instr_val.get_instruction(self.context) {
            match instruction {
                Instruction::AsmBlock(asm, args) => todo!(),
                Instruction::BitCast(val, ty) => todo!(),
                Instruction::BinaryOp { op, arg1, arg2 } => {
                    todo!()
                }
                Instruction::Branch(to_block) => todo!(),
                Instruction::Call(func, args) => self.compile_call(instr_val, func, args),
                Instruction::CastPtr(val, ty) => {
                    todo!()
                }
                Instruction::Cmp(pred, lhs_value, rhs_value) => {
                    todo!()
                }
                Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.compile_conditional_branch(cond_value, true_block, false_block),
                Instruction::ContractCall {
                    params,
                    coins,
                    asset_id,
                    gas,
                    ..
                } => todo!(),
                Instruction::FuelVm(fuel_vm_instr) => todo!(),
                Instruction::GetElemPtr {
                    base,
                    elem_ptr_ty,
                    indices,
                } => todo!(),
                Instruction::GetLocal(local_var) => todo!(),
                Instruction::IntToPtr(val, _) => todo!(),
                Instruction::Load(src_val) => todo!(),
                Instruction::MemCopyBytes {
                    dst_val_ptr,
                    src_val_ptr,
                    byte_len,
                } => todo!(),
                Instruction::MemCopyVal {
                    dst_val_ptr,
                    src_val_ptr,
                } => todo!(),
                Instruction::Nop => (),
                Instruction::PtrToInt(ptr_val, int_ty) => todo!(),
                Instruction::Ret(ret_val, ty) => self.compile_return(ret_val, ty),
                Instruction::Store {
                    dst_val_ptr,
                    stored_val,
                } => todo!(),
            }
        } else {
            panic!(
                "{:?}",
                CompileError::Internal(
                    "Value not an instruction.",
                    self.md_mgr
                        .val_to_span(self.context, *instr_val)
                        .unwrap_or_else(Self::empty_span),
                )
            )
        }
    }

    fn compile_asm_block(
        &mut self,
        instr_val: &Value,
        asm: &AsmBlock,
        asm_args: &[AsmArg],
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_addr_of(&mut self, instr_val: &Value, arg: &Value) {
        todo!();
    }

    fn compile_bitcast(&mut self, instr_val: &Value, bitcast_val: &Value, to_type: &Type) {
        todo!();
    }

    fn compile_binary_op(
        &mut self,
        instr_val: &Value,
        op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
    ) {
        todo!();
    }

    fn compile_cast_ptr(&mut self, instr_val: &Value, val: &Value, ty: &Type, offs: u64) {
        todo!();
    }

    fn compile_cmp(
        &mut self,
        instr_val: &Value,
        pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) {
        todo!();
    }

    fn compile_branch_to_phi_value(&mut self, to_block: &BranchToWithArgs) {
        todo!();
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        instr_val: &Value,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) {
        todo!();
    }

    fn compile_extract_element(
        &mut self,
        instr_val: &Value,
        array: &Value,
        ty: &Type,
        index_val: &Value,
    ) {
        todo!();
    }

    fn compile_extract_value(&mut self, instr_val: &Value, aggregate_val: &Value, indices: &[u64]) {
        todo!();
    }

    fn compile_get_storage_key(&mut self, instr_val: &Value) -> CompileResult<()> {
        todo!();
    }

    fn compile_get_local(&mut self, instr_val: &Value, local_var: &LocalVar) {
        todo!();
    }

    fn compile_gtf(&mut self, instr_val: &Value, index: &Value, tx_field_id: u64) {
        todo!();
    }

    fn compile_insert_element(
        &mut self,
        instr_val: &Value,
        array: &Value,
        ty: &Type,
        value: &Value,
        index_val: &Value,
    ) {
        todo!();
    }

    fn compile_insert_value(
        &mut self,
        instr_val: &Value,
        aggregate_val: &Value,
        value: &Value,
        indices: &[u64],
    ) {
        todo!();
    }

    fn compile_int_to_ptr(&mut self, instr_val: &Value, int_to_ptr_val: &Value) {
        todo!();
    }

    fn compile_load(&mut self, instr_val: &Value, src_val: &Value) -> CompileResult<()> {
        todo!();
    }

    fn compile_mem_copy(
        &mut self,
        instr_val: &Value,
        dst_val: &Value,
        src_val: &Value,
        byte_len: u64,
    ) {
        todo!();
    }

    fn compile_log(&mut self, instr_val: &Value, log_val: &Value, log_ty: &Type, log_id: &Value) {
        todo!();
    }

    fn compile_read_register(&mut self, instr_val: &Value, reg: &sway_ir::Register) {
        todo!();
    }

    fn compile_ret_from_entry(&mut self, instr_val: &Value, ret_val: &Value, ret_type: &Type) {
        todo!();
    }

    fn compile_revert(&mut self, instr_val: &Value, revert_val: &Value) {
        todo!();
    }

    fn compile_smo(
        &mut self,
        instr_val: &Value,
        recipient_and_message: &Value,
        message_size: &Value,
        output_index: &Value,
        coins: &Value,
    ) {
        todo!();
    }

    fn compile_state_access_quad_word(
        &mut self,
        instr_val: &Value,
        val: &Value,
        key: &Value,
        number_of_slots: &Value,
        access_type: StateAccessType,
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_state_load_word(&mut self, instr_val: &Value, key: &Value) -> CompileResult<()> {
        todo!();
    }

    fn compile_state_store_word(
        &mut self,
        instr_val: &Value,
        store_val: &Value,
        key: &Value,
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_store(
        &mut self,
        instr_val: &Value,
        dst_val: &Value,
        stored_val: &Value,
    ) -> CompileResult<()> {
        todo!();
    }

    pub fn compile_function(&mut self, function: Function) -> CompileResult<()> {
        if function.get_name(self.context).to_lowercase() != "main" {
            self.set_active_procedure(&function);
        }
        self.compile_code_block(function.block_iter(self.context));
        self.end_active_procedure();
        ok((), vec![], vec![])
    }

    fn compile_code_block(&mut self, block: BlockIterator) {
        for block in block {
            for instr_val in block.instruction_iter(self.context) {
                self.compile_instruction(&instr_val);
            }
        }
    }

    pub(super) fn compile_ret_from_call(&mut self, instr_val: &Value, ret_val: &Value) {
        todo!();
    }

    /// Compiles a return instruction.
    /// https://0xpolygonmiden.github.io/miden-vm/design/stack/main.html#operand-stack
    /// By the end of program execution, exactly 16 items must remain on the stack (again,
    ///  all of them could be 0's). These items comprise the output of the program.

    /// all we have to do here is make sure the returned value is on top of the stack
    /// this works in both internal procedure returns and external main returns
    pub fn compile_return(&mut self, ret_val: &Value, ty: &Type) {
        self.compile_value_access(ret_val)
    }

    fn compile_value_access(&mut self, ret_val: &Value) {
        self.push_op(MidenAsmOp::access_value(*ret_val));
    }

    fn push_op(&mut self, op: MidenAsmOp) {
        if let Some(ref proc_name) = self.active_procedure {
            let res = self.procedure_map.get_mut(proc_name);
            if let Some(res) = res {
                res.ops.push(op);
            } else {
                unreachable!("It is an invariant that a proc name would be in the mapping already. This is an internal compiler error.")
            }
        } else {
            self.buf.push(op);
        }
    }

    fn set_active_procedure(&mut self, function: &Function) {
        self.procedure_map.insert(
            function.get_name(self.context).into(),
            Procedure {
                number_of_locals: function.num_args(self.context) as u32,
                ops: Default::default(),
            },
        );
    }

    fn end_active_procedure(&mut self) {
        self.active_procedure = None;
    }

    fn render_abstract_op(&self, op: &miden_op::AbstractOp) -> Vec<DirectOp> {
        use miden_op::AbstractOp::*;
        match op {
            AccessValue(val) => {
                // TODO: configurables, instructions, etc
                //        todo!("check chained or-else stuff in fuel asm builder")
                let rendered = val
                    .get_constant(self.context)
                    .map(|constant| self.render_constant(constant))
                    .or_else(|| val.get_configurable(self.context).map(|config| todo!()))
                    .or_else(|| todo!());
                if let Some(rendered) = rendered {
                    rendered
                } else {
                    panic!("Not sure what this value is -- is'nt a constant or a configurable. {val:?}")
                }
            }
        }
    }

    fn render_op(&self, op: &MidenAsmOp) -> Vec<DirectOp> {
        match op {
            MidenAsmOp::DirectOp(a) => vec![a.clone()],
            MidenAsmOp::AbstractOp(op) => self.render_abstract_op(op),
        }
    }

    /// Pushes a constant to the top of the stack
    pub(crate) fn render_constant(&self, constant: &Constant) -> Vec<DirectOp> {
        use sway_ir::ConstantValue::*;
        match constant.value {
            Undef => todo!(),
            Unit => vec![DirectOp::push(MidenStackValue::Unit)],
            Bool(b) => vec![DirectOp::push(b)],
            Uint(x) => vec![DirectOp::push(x)],
            B256(_) => todo!(),
            String(_) => todo!(),
            Array(_) => todo!(),
            Struct(_) => todo!(),
        }
    }
}

pub trait ToMidenBytecode {
    // the midenVM expects assembly in String form
    fn to_bytecode(&self) -> String;
}

impl ToMidenBytecode for Vec<DirectOp> {
    fn to_bytecode(&self) -> String {
        // TODO nice indentation and stuff
        self.iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
