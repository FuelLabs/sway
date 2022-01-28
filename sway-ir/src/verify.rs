//! Code to validate the IR in a [`Context`].
//!
//! During creation, deserialization and optimization the IR should be verified to be in a
//! consistent valid state, using the functions in this module.
//!
//! **NOTE: this module is very much a WIP.  I began to add verifications only for the IR to change
//! and make them obsolete or outdated.  So once the IR is in a stable state and in use in the Sway
//! compiler, this module must be updated and used.**

use std::iter::FromIterator;

use crate::{
    asm::{AsmArg, AsmBlock},
    block::{Block, BlockContent},
    context::Context,
    function::{Function, FunctionContent},
    instruction::Instruction,
    irtype::{Aggregate, Type},
    module::ModuleContent,
    pointer::Pointer,
    value::{Value, ValueContent},
};

impl Context {
    /// Verify the contents of this [`Context`] is valid.
    pub fn verify(&self) -> Result<(), String> {
        for (_, module) in &self.modules {
            self.verify_module(module)?;
        }
        Ok(())
    }

    fn verify_module(&self, module: &ModuleContent) -> Result<(), String> {
        for function in &module.functions {
            self.verify_function(&self.functions[function.0])?;
        }
        Ok(())
    }

    fn verify_function(&self, function: &FunctionContent) -> Result<(), String> {
        for block in &function.blocks {
            self.verify_block(function, &self.blocks[block.0])?;
        }
        Ok(())
    }

    fn verify_block(&self, function: &FunctionContent, block: &BlockContent) -> Result<(), String> {
        for ins in &block.instructions {
            self.verify_instruction(function, &self.values[ins.0])?;
        }
        let (last_is_term, num_terms) =
            block.instructions.iter().fold((false, 0), |(_, n), ins| {
                if ins.is_terminator(self) {
                    (true, n + 1)
                } else {
                    (false, n)
                }
            });
        if !last_is_term || num_terms != 1 {
            Err(format!(
                "Block {} must have single terminator as its last instruction.\n\n{}",
                block.label, self
            ))
        } else {
            Ok(())
        }
    }

    fn verify_instruction(
        &self,
        function: &FunctionContent,
        instruction: &ValueContent,
    ) -> Result<(), String> {
        if let ValueContent::Instruction(instruction) = instruction {
            match instruction {
                Instruction::AsmBlock(asm, args) => self.verify_asm_block(asm, args)?,
                Instruction::Branch(block) => self.verify_br(block)?,
                Instruction::Call(func, args) => self.verify_call(func, args)?,
                Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.verify_cbr(cond_value, true_block, false_block)?,
                Instruction::ExtractElement {
                    array,
                    ty,
                    index_val,
                } => self.verify_extract_element(array, ty, index_val)?,
                Instruction::ExtractValue {
                    aggregate,
                    ty,
                    indices,
                } => self.verify_extract_value(aggregate, ty, indices)?,
                Instruction::GetPointer(ptr) => self.verify_get_ptr(ptr)?,
                Instruction::InsertElement {
                    array,
                    ty,
                    value,
                    index_val,
                } => self.verify_insert_element(array, ty, value, index_val)?,
                Instruction::InsertValue {
                    aggregate,
                    ty,
                    value,
                    indices,
                } => self.verify_insert_values(aggregate, ty, value, indices)?,
                Instruction::Load(ptr) => self.verify_load(ptr)?,
                Instruction::Phi(pairs) => self.verify_phi(&pairs[..])?,
                Instruction::Ret(val, ty) => self.verify_ret(function, val, ty)?,
                Instruction::Store { ptr, stored_val } => self.verify_store(ptr, stored_val)?,
            }
        } else {
            unreachable!("Verify instruction is not an instruction.");
        }
        Ok(())
    }

    fn verify_asm_block(&self, _asm: &AsmBlock, _args: &[AsmArg]) -> Result<(), String> {
        Ok(())
    }

    fn verify_br(&self, _block: &Block) -> Result<(), String> {
        Ok(())
    }

    fn verify_call(&self, _callee: &Function, _args: &[Value]) -> Result<(), String> {
        // XXX We should confirm the function arg types are all correct and the return type matches
        // the call value type... but all they type info isn't stored at this stage, and it
        // should've all been checked in the typed AST.
        Ok(())
    }

    fn verify_cbr(
        &self,
        _cond_val: &Value,
        _true_block: &Block,
        _false_block: &Block,
    ) -> Result<(), String> {
        // XXX When we have some type info available from instructions...
        //if !cond_val.is_bool_ty(self) {
        //    Err("Condition for branch must be a bool.".into())
        //} else {
        Ok(())
        //}
    }

    fn verify_extract_element(
        &self,
        _array: &Value,
        _ty: &Aggregate,
        _index_val: &Value,
    ) -> Result<(), String> {
        Ok(())
    }

    fn verify_extract_value(
        &self,
        _aggregate: &Value,
        _ty: &Aggregate,
        _indices: &[u64],
    ) -> Result<(), String> {
        // XXX Are we checking the context knows about the aggregate and the indices are valid?  Or
        // is that the type checker's problem?
        Ok(())
    }

    fn verify_get_ptr(&self, _ptr: &Pointer) -> Result<(), String> {
        // XXX get_ptr() shouldn't exist in the final IR?
        Ok(())
    }

    fn verify_insert_element(
        &self,
        _array: &Value,
        _ty: &Aggregate,
        _value: &Value,
        _index_val: &Value,
    ) -> Result<(), String> {
        Ok(())
    }

    fn verify_insert_values(
        &self,
        _aggregate: &Value,
        _ty: &Aggregate,
        _value: &Value,
        _idcs: &[u64],
    ) -> Result<(), String> {
        // XXX The types should all line up.
        Ok(())
    }

    fn verify_load(&self, _ptr: &Pointer) -> Result<(), String> {
        // XXX We should check the pointer type matches this load type.
        Ok(())
    }

    fn verify_phi(&self, pairs: &[(Block, Value)]) -> Result<(), String> {
        let label_set = std::collections::HashSet::<&String>::from_iter(
            pairs.iter().map(|(block, _)| &(self.blocks[block.0].label)),
        );
        if label_set.len() != pairs.len() {
            Err("Phi must have unique block labels.".into())
        } else {
            Ok(())
        }
    }

    fn verify_ret(
        &self,
        function: &FunctionContent,
        _val: &Value,
        ty: &Type,
    ) -> Result<(), String> {
        if &function.return_type != ty {
            println!("{:?} != {:?}", &function.return_type, ty);
            Err(format!(
                "Function {} return type must match ret instructions.",
                function.name
            ))
        // XXX When we have some type info available from instructions...
        //} else if val.get_type(self) != Some(*ty) {
        //    Err("Ret value type must match return type.".into())
        } else {
            Ok(())
        }
    }

    fn verify_store(&self, _ptr: &Pointer, _stored_val: &Value) -> Result<(), String> {
        // XXX When we have some type info available from instructions...
        //if ptr_val.get_type(self) != stored_val.get_type(self) {
        //    Err("Stored value type must match pointer type.".into())
        //} else {
        Ok(())
        //}
    }
}
