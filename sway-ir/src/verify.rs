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
    error::IrError,
    function::{Function, FunctionContent},
    instruction::Instruction,
    irtype::{Aggregate, Type},
    module::ModuleContent,
    pointer::Pointer,
    value::{Value, ValueDatum},
};

impl Context {
    /// Verify the contents of this [`Context`] is valid.
    pub fn verify(&self) -> Result<(), IrError> {
        for (_, module) in &self.modules {
            self.verify_module(module)?;
        }
        Ok(())
    }

    fn verify_module(&self, module: &ModuleContent) -> Result<(), IrError> {
        for function in &module.functions {
            self.verify_function(&self.functions[function.0])?;
        }
        Ok(())
    }

    fn verify_function(&self, function: &FunctionContent) -> Result<(), IrError> {
        for block in &function.blocks {
            self.verify_block(function, &self.blocks[block.0])?;
        }
        Ok(())
    }

    fn verify_block(
        &self,
        function: &FunctionContent,
        block: &BlockContent,
    ) -> Result<(), IrError> {
        for ins in &block.instructions {
            self.verify_instruction(function, &self.values[ins.0].value)?;
        }
        let (last_is_term, num_terms) =
            block.instructions.iter().fold((false, 0), |(_, n), ins| {
                if ins.is_terminator(self) {
                    (true, n + 1)
                } else {
                    (false, n)
                }
            });
        if !last_is_term {
            Err(IrError::MissingTerminator(block.label.clone()))
        } else if num_terms != 1 {
            Err(IrError::MisplacedTerminator(block.label.clone()))
        } else {
            Ok(())
        }
    }

    fn verify_instruction(
        &self,
        function: &FunctionContent,
        instruction: &ValueDatum,
    ) -> Result<(), IrError> {
        if let ValueDatum::Instruction(instruction) = instruction {
            match instruction {
                Instruction::AsmBlock(asm, args) => self.verify_asm_block(asm, args)?,
                Instruction::Branch(block) => self.verify_br(block)?,
                Instruction::Call(func, args) => self.verify_call(func, args)?,
                Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.verify_cbr(cond_value, true_block, false_block)?,
                Instruction::ContractCall {
                    params,
                    coins,
                    asset_id,
                    gas,
                } => self.verify_contract_call(params, coins, asset_id, gas)?,
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
                Instruction::GetPointer {
                    base_ptr,
                    ptr_ty,
                    offset,
                } => self.verify_get_ptr(base_ptr, ptr_ty, *offset)?,
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
                Instruction::Nop => (),
                Instruction::Phi(pairs) => self.verify_phi(&pairs[..])?,
                Instruction::ReadRegister { reg_name } => self.verify_read_register(reg_name)?,
                Instruction::Ret(val, ty) => self.verify_ret(function, val, ty)?,
                Instruction::StateLoadWord(key) => self.verify_state_load_word(key)?,
                Instruction::StateLoadQuadWord { load_val, key } => {
                    self.verify_state_load_quad_word(load_val, key)?
                }
                Instruction::StateStoreWord { stored_val, key } => {
                    self.verify_state_store_word(stored_val, key)?
                }
                Instruction::StateStoreQuadWord { stored_val, key } => {
                    self.verify_state_store_quad_word(stored_val, key)?
                }
                Instruction::Store {
                    dst_val,
                    stored_val,
                } => self.verify_store(dst_val, stored_val)?,
            }
        } else {
            unreachable!("Verify instruction is not an instruction.");
        }
        Ok(())
    }

    fn verify_asm_block(&self, _asm: &AsmBlock, _args: &[AsmArg]) -> Result<(), IrError> {
        Ok(())
    }

    fn verify_br(&self, _block: &Block) -> Result<(), IrError> {
        Ok(())
    }

    fn verify_call(&self, _callee: &Function, _args: &[Value]) -> Result<(), IrError> {
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
    ) -> Result<(), IrError> {
        // XXX When we have some type info available from instructions...
        //if !cond_val.is_bool_ty(self) {
        //    Err("Condition for branch must be a bool.".into())
        //} else {
        Ok(())
        //}
    }

    #[allow(clippy::too_many_arguments)]
    fn verify_contract_call(
        &self,
        _params: &Value,
        _coins: &Value,
        _asset_id: &Value,
        _gas: &Value,
    ) -> Result<(), IrError> {
        // XXX We should confirm the function arg types and the return type are all correct.
        // We should also check that addr comes from a b256 local pointer and that coins and gas
        // are u64 values, while asset_id is a b256.
        Ok(())
    }

    fn verify_extract_element(
        &self,
        _array: &Value,
        _ty: &Aggregate,
        _index_val: &Value,
    ) -> Result<(), IrError> {
        Ok(())
    }

    fn verify_extract_value(
        &self,
        _aggregate: &Value,
        _ty: &Aggregate,
        _indices: &[u64],
    ) -> Result<(), IrError> {
        // XXX Are we checking the context knows about the aggregate and the indices are valid?  Or
        // is that the type checker's problem?
        Ok(())
    }

    fn verify_get_ptr(
        &self,
        _base_ptr: &Pointer,
        _ptr_ty: &Type,
        _offset: u64,
    ) -> Result<(), IrError> {
        // XXX get_ptr() shouldn't exist in the final IR?
        Ok(())
    }

    fn verify_insert_element(
        &self,
        _array: &Value,
        _ty: &Aggregate,
        _value: &Value,
        _index_val: &Value,
    ) -> Result<(), IrError> {
        Ok(())
    }

    fn verify_insert_values(
        &self,
        _aggregate: &Value,
        _ty: &Aggregate,
        _value: &Value,
        _idcs: &[u64],
    ) -> Result<(), IrError> {
        // XXX The types should all line up.
        Ok(())
    }

    fn verify_load(&self, _src_val: &Value) -> Result<(), IrError> {
        // XXX src_val must be a pointer.
        // XXX We should check the pointer type matches this load type.
        Ok(())
    }

    fn verify_phi(&self, pairs: &[(Block, Value)]) -> Result<(), IrError> {
        let label_set = std::collections::HashSet::<&String>::from_iter(
            pairs.iter().map(|(block, _)| &(self.blocks[block.0].label)),
        );
        if label_set.len() != pairs.len() {
            Err(IrError::NonUniquePhiLabels)
        } else {
            Ok(())
        }
    }

    fn verify_read_register(&self, _reg: &str) -> Result<(), IrError> {
        // We may want to verify that the register passed actually exists
        Ok(())
    }

    fn verify_ret(
        &self,
        function: &FunctionContent,
        _val: &Value,
        ty: &Type,
    ) -> Result<(), IrError> {
        if &function.return_type != ty {
            println!("{:?} != {:?}", &function.return_type, ty);
            Err(IrError::MismatchedReturnTypes(function.name.clone()))
        // XXX When we have some type info available from instructions...
        //} else if val.get_type(self) != Some(*ty) {
        //    Err("Ret value type must match return type.".into())
        } else {
            Ok(())
        }
    }

    fn verify_state_load_quad_word(&self, _load_val: &Value, _key: &Value) -> Result<(), IrError> {
        // XXX key must be a pointer to B256, load_val ty must by pointer to either Uint(64) or B256.
        Ok(())
    }

    fn verify_state_load_word(&self, _key: &Value) -> Result<(), IrError> {
        // XXX key must be a pointer to B256, load_val ty must by pointer to either Uint(64) or B256.
        Ok(())
    }

    fn verify_state_store_quad_word(
        &self,
        _stored_val: &Value,
        _key: &Value,
    ) -> Result<(), IrError> {
        // XXX key must be a pointer to B256, stored val ty must be pointer to a B256.
        Ok(())
    }

    fn verify_state_store_word(&self, _stored_val: &Value, _key: &Value) -> Result<(), IrError> {
        // XXX key must be a pointer to B256, stored val ty must be a Uint(64).
        Ok(())
    }

    fn verify_store(&self, _dst_val: &Value, _stored_val: &Value) -> Result<(), IrError> {
        // XXX When we have some type info available from instructions...
        // XXX dst must be a pointer.
        // XXX Pointer destinations must be mutable.
        //if ptr_val.get_type(self) != stored_val.get_type(self) {
        //    Err("Stored value type must match pointer type.".into())
        //} else {
        Ok(())
        //}
    }
}
