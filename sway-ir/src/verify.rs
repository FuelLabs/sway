//! Code to validate the IR in a [`Context`].
//!
//! During creation, deserialization and optimization the IR should be verified to be in a
//! consistent valid state, using the functions in this module.

use std::iter::FromIterator;

use crate::{
    block::{Block, BlockContent},
    context::Context,
    error::IrError,
    function::{Function, FunctionContent},
    instruction::{Instruction, Predicate},
    irtype::{Aggregate, Type},
    metadata::{MetadataIndex, Metadatum},
    module::ModuleContent,
    pointer::Pointer,
    value::{Value, ValueDatum},
};

impl Context {
    /// Verify the contents of this [`Context`] is valid.
    pub fn verify(self) -> Result<Self, IrError> {
        for (_, module) in &self.modules {
            self.verify_module(module)?;
        }
        Ok(self)
    }

    fn verify_module(&self, module: &ModuleContent) -> Result<(), IrError> {
        for function in &module.functions {
            self.verify_function(module, &self.functions[function.0])?;
        }
        Ok(())
    }

    fn verify_function(
        &self,
        cur_module: &ModuleContent,
        function: &FunctionContent,
    ) -> Result<(), IrError> {
        for block in &function.blocks {
            self.verify_block(cur_module, function, &self.blocks[block.0])?;
        }
        self.verify_metadata(function.metadata)?;
        Ok(())
    }

    fn verify_block(
        &self,
        cur_module: &ModuleContent,
        cur_function: &FunctionContent,
        block: &BlockContent,
    ) -> Result<(), IrError> {
        if block.instructions.len() <= 1 && block.num_predecessors(self) == 0 {
            // Empty (containing only the phi) unreferenced blocks are a harmless artefact.
            return Ok(());
        }

        InstructionVerifier {
            context: self,
            cur_module,
            cur_function,
            cur_block: block,
        }
        .verify_instructions()?;

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

    fn verify_metadata(&self, md_idx: Option<MetadataIndex>) -> Result<(), IrError> {
        // For now we check only that struct tags are valid identiers.
        if let Some(md_idx) = md_idx {
            match &self.metadata[md_idx.0] {
                Metadatum::List(md_idcs) => {
                    for md_idx in md_idcs {
                        self.verify_metadata(Some(*md_idx))?;
                    }
                }
                Metadatum::Struct(tag, ..) => {
                    // We could import Regex to match it, but it's a simple identifier style pattern:
                    // alpha start char, alphanumeric for the rest, or underscore anywhere.
                    if tag.is_empty() {
                        return Err(IrError::InvalidMetadatum(
                            "Struct has empty tag.".to_owned(),
                        ));
                    }
                    let mut chs = tag.chars();
                    let ch0 = chs.next().unwrap();
                    if !(ch0.is_ascii_alphabetic() || ch0 == '_')
                        || chs.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    {
                        return Err(IrError::InvalidMetadatum(format!(
                            "Invalid struct tag: '{tag}'."
                        )));
                    }
                }
                _otherwise => (),
            }
        }
        Ok(())
    }
}

struct InstructionVerifier<'a> {
    context: &'a Context,
    cur_module: &'a ModuleContent,
    cur_function: &'a FunctionContent,
    cur_block: &'a BlockContent,
}

impl<'a> InstructionVerifier<'a> {
    fn verify_instructions(&self) -> Result<(), IrError> {
        for ins in &self.cur_block.instructions {
            let value_content = &self.context.values[ins.0];
            if let ValueDatum::Instruction(instruction) = &value_content.value {
                match instruction {
                    Instruction::AddrOf(arg) => self.verify_addr_of(arg)?,
                    Instruction::AsmBlock(..) => (),
                    Instruction::BitCast(value, ty) => self.verify_bitcast(value, ty)?,
                    Instruction::Branch(block) => self.verify_br(block)?,
                    Instruction::Call(func, args) => self.verify_call(func, args)?,
                    Instruction::Cmp(pred, lhs_value, rhs_value) => {
                        self.verify_cmp(pred, lhs_value, rhs_value)?
                    }
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
                        ..
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
                    Instruction::GetStorageKey => (),
                    Instruction::GetPointer {
                        base_ptr,
                        ptr_ty,
                        offset,
                    } => self.verify_get_ptr(base_ptr, ptr_ty, offset)?,
                    Instruction::Gtf { index, tx_field_id } => {
                        self.verify_gtf(index, tx_field_id)?
                    }
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
                    } => self.verify_insert_value(aggregate, ty, value, indices)?,
                    Instruction::IntToPtr(value, ty) => self.verify_int_to_ptr(value, ty)?,
                    Instruction::Load(ptr) => self.verify_load(ptr)?,
                    Instruction::Nop => (),
                    Instruction::Phi(pairs) => self.verify_phi(&pairs[..])?,
                    Instruction::ReadRegister(_) => (),
                    Instruction::Ret(val, ty) => self.verify_ret(self.cur_function, val, ty)?,
                    Instruction::StateLoadWord(key) => self.verify_state_load_word(key)?,
                    Instruction::StateLoadQuadWord {
                        load_val: dst_val,
                        key,
                    }
                    | Instruction::StateStoreQuadWord {
                        stored_val: dst_val,
                        key,
                    } => self.verify_state_load_store(dst_val, &Type::B256, key)?,
                    Instruction::StateStoreWord {
                        stored_val: dst_val,
                        key,
                    } => self.verify_state_store_word(dst_val, key)?,
                    Instruction::Store {
                        dst_val,
                        stored_val,
                    } => self.verify_store(dst_val, stored_val)?,
                };

                // Verify the instruction metadata too.
                self.context.verify_metadata(value_content.metadata)?;
            } else {
                unreachable!("Verify instruction is not an instruction.");
            }
        }
        Ok(())
    }

    fn verify_addr_of(&self, value: &Value) -> Result<(), IrError> {
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyAddrOfUnknownSourceType)?;
        if val_ty.is_copy_type() {
            return Err(IrError::VerifyAddrOfCopyType);
        }
        Ok(())
    }

    fn verify_bitcast(&self, value: &Value, ty: &Type) -> Result<(), IrError> {
        // The to and from types must be copy-types, excluding short strings, and the same size.
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyBitcastUnknownSourceType)?;
        if !val_ty.is_copy_type() {
            return Err(IrError::VerifyBitcastFromNonCopyType(
                val_ty.as_string(self.context),
            ));
        }
        if !ty.is_copy_type() {
            return Err(IrError::VerifyBitcastToNonCopyType(
                val_ty.as_string(self.context),
            ));
        }
        let is_valid = match val_ty {
            Type::Unit | Type::Bool => true, // Unit or bool to any copy type works.
            Type::Uint(from_nbits) => match ty {
                Type::Unit | Type::Bool => true, // We can construct a unit or bool from any sized integer.
                Type::Uint(to_nbits) => from_nbits == *to_nbits,
                _otherwise => false,
            },
            Type::B256 | Type::String(_) | Type::Array(_) | Type::Union(_) | Type::Struct(_) => {
                false
            }
        };
        if !is_valid {
            Err(IrError::VerifyBitcastBetweenInvalidTypes(
                val_ty.as_string(self.context),
                ty.as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_br(&self, dest_block: &Block) -> Result<(), IrError> {
        if !self.cur_function.blocks.contains(dest_block) {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[dest_block.0].label.clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_call(&self, callee: &Function, args: &[Value]) -> Result<(), IrError> {
        let callee_content = &self.context.functions[callee.0];
        if !self.cur_module.functions.contains(callee) {
            Err(IrError::VerifyCallToMissingFunction(
                callee_content.name.clone(),
            ))
        } else {
            let callee_arg_types = callee_content
                .arguments
                .iter()
                .map(|(_, arg_val)| {
                    if let ValueDatum::Argument(ty) = &self.context.values[arg_val.0].value {
                        Ok(*ty)
                    } else {
                        Err(IrError::VerifyArgumentValueIsNotArgument(
                            callee_content.name.clone(),
                        ))
                    }
                })
                .collect::<Result<Vec<Type>, IrError>>()?;

            for (opt_caller_arg_type, callee_arg_type) in args
                .iter()
                .map(|val| val.get_type(self.context))
                .zip(callee_arg_types.iter())
            {
                if opt_caller_arg_type.is_none() {
                    return Err(IrError::VerifyUntypedValuePassedToFunction);
                }
                if !opt_caller_arg_type
                    .as_ref()
                    .unwrap()
                    .eq(self.context, callee_arg_type)
                {
                    return Err(IrError::VerifyCallArgTypeMismatch(
                        callee_content.name.clone(),
                    ));
                }
            }

            Ok(())
        }
    }

    fn verify_cbr(
        &self,
        cond_val: &Value,
        true_block: &Block,
        false_block: &Block,
    ) -> Result<(), IrError> {
        if !matches!(cond_val.get_type(self.context), Some(Type::Bool)) {
            Err(IrError::VerifyConditionExprNotABool)
        } else if !self.cur_function.blocks.contains(true_block) {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[true_block.0].label.clone(),
            ))
        } else if !self.cur_function.blocks.contains(false_block) {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[false_block.0].label.clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_cmp(
        &self,
        _pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) -> Result<(), IrError> {
        // Comparisons must be between integers at this stage.
        match (
            lhs_value.get_type(self.context),
            rhs_value.get_type(self.context),
        ) {
            (Some(lhs_ty), Some(rhs_ty)) => match (lhs_ty, rhs_ty) {
                (Type::Uint(lhs_nbits), Type::Uint(rhs_nbits)) => {
                    if lhs_nbits != rhs_nbits {
                        Err(IrError::VerifyCmpTypeMismatch(
                            lhs_ty.as_string(self.context),
                            rhs_ty.as_string(self.context),
                        ))
                    } else {
                        Ok(())
                    }
                }
                (Type::Bool, Type::Bool) => Ok(()),
                _otherwise => Err(IrError::VerifyCmpBadTypes(
                    lhs_ty.as_string(self.context),
                    rhs_ty.as_string(self.context),
                )),
            },
            _otherwise => Err(IrError::VerifyCmpUnknownTypes),
        }
    }

    fn verify_contract_call(
        &self,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) -> Result<(), IrError> {
        // - The params must be a struct with the B256 address, u64 selector and u64 address to
        //   user args.
        // - The coins and gas must be u64s.
        // - The asset_id must be a B256
        if let Some(Type::Struct(agg)) = params.get_type(self.context) {
            let fields = self.context.aggregates[agg.0].field_types();
            if fields.len() != 3
                || !fields[0].eq(self.context, &Type::B256)
                || !fields[1].eq(self.context, &Type::Uint(64))
                || !fields[2].eq(self.context, &Type::Uint(64))
            {
                Err(IrError::VerifyContractCallBadTypes("params".to_owned()))
            } else {
                Ok(())
            }
        } else {
            Err(IrError::VerifyContractCallBadTypes("params".to_owned()))
        }
        .and_then(|_| {
            if let Some(Type::Uint(64)) = coins.get_type(self.context) {
                Ok(())
            } else {
                Err(IrError::VerifyContractCallBadTypes("coins".to_owned()))
            }
        })
        .and_then(|_| {
            if let Some(Type::B256) = asset_id.get_type(self.context) {
                Ok(())
            } else {
                Err(IrError::VerifyContractCallBadTypes("asset_id".to_owned()))
            }
        })
        .and_then(|_| {
            if let Some(Type::Uint(64)) = gas.get_type(self.context) {
                Ok(())
            } else {
                Err(IrError::VerifyContractCallBadTypes("gas".to_owned()))
            }
        })
    }

    fn verify_extract_element(
        &self,
        array: &Value,
        ty: &Aggregate,
        index_val: &Value,
    ) -> Result<(), IrError> {
        match array.get_type(self.context) {
            Some(Type::Array(ary_ty)) => {
                if !ary_ty.is_equivalent(self.context, ty) {
                    Err(IrError::VerifyAccessElementInconsistentTypes)
                } else if !matches!(index_val.get_type(self.context), Some(Type::Uint(_))) {
                    Err(IrError::VerifyAccessElementNonIntIndex)
                } else {
                    Ok(())
                }
            }
            _otherwise => Err(IrError::VerifyAccessElementOnNonArray),
        }
    }

    fn verify_extract_value(
        &self,
        aggregate: &Value,
        ty: &Aggregate,
        indices: &[u64],
    ) -> Result<(), IrError> {
        match aggregate.get_type(self.context) {
            Some(Type::Struct(agg_ty)) | Some(Type::Union(agg_ty)) => {
                if !agg_ty.is_equivalent(self.context, ty) {
                    Err(IrError::VerifyAccessValueInconsistentTypes)
                } else if ty.get_field_type(self.context, indices).is_none() {
                    Err(IrError::VerifyAccessValueInvalidIndices)
                } else {
                    Ok(())
                }
            }
            _otherwise => Err(IrError::VerifyAccessValueOnNonStruct),
        }
    }

    fn verify_get_ptr(
        &self,
        base_ptr: &Pointer,
        _ptr_ty: &Type,
        _offset: &u64,
    ) -> Result<(), IrError> {
        // We should perhaps verify that the offset and the casted type fit within the base type.
        if !self.is_local_pointer(base_ptr) {
            Err(IrError::VerifyGetNonExistentPointer)
        } else {
            Ok(())
        }
    }

    fn verify_gtf(&self, index: &Value, _tx_field_id: &u64) -> Result<(), IrError> {
        // We should perhaps verify that _tx_field_id fits in a twelve bit immediate
        if !matches!(index.get_type(self.context), Some(Type::Uint(_))) {
            Err(IrError::VerifyInvalidGtfIndexType)
        } else {
            Ok(())
        }
    }

    fn verify_insert_element(
        &self,
        array: &Value,
        ty: &Aggregate,
        value: &Value,
        index_val: &Value,
    ) -> Result<(), IrError> {
        match array.get_type(self.context) {
            Some(Type::Array(ary_ty)) => {
                if !ary_ty.is_equivalent(self.context, ty) {
                    Err(IrError::VerifyAccessElementInconsistentTypes)
                } else if self.opt_ty_not_eq(
                    &ty.get_elem_type(self.context),
                    &value.get_type(self.context),
                ) {
                    Err(IrError::VerifyInsertElementOfIncorrectType)
                } else if !matches!(index_val.get_type(self.context), Some(Type::Uint(_))) {
                    Err(IrError::VerifyAccessElementNonIntIndex)
                } else {
                    Ok(())
                }
            }
            _otherwise => Err(IrError::VerifyAccessElementOnNonArray),
        }
    }

    fn verify_insert_value(
        &self,
        aggregate: &Value,
        ty: &Aggregate,
        value: &Value,
        idcs: &[u64],
    ) -> Result<(), IrError> {
        match aggregate.get_type(self.context) {
            Some(Type::Struct(str_ty)) => {
                if !str_ty.is_equivalent(self.context, ty) {
                    Err(IrError::VerifyAccessValueInconsistentTypes)
                } else {
                    let field_ty = ty.get_field_type(self.context, idcs);
                    if field_ty.is_none() {
                        Err(IrError::VerifyAccessValueInvalidIndices)
                    } else if self.opt_ty_not_eq(&field_ty, &value.get_type(self.context)) {
                        Err(IrError::VerifyInsertValueOfIncorrectType)
                    } else {
                        Ok(())
                    }
                }
            }
            _otherwise => Err(IrError::VerifyAccessValueOnNonStruct),
        }
    }

    fn verify_int_to_ptr(&self, value: &Value, ty: &Type) -> Result<(), IrError> {
        // We want the source value to be an integer and the destination type to be a reference
        // type.
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyIntToPtrUnknownSourceType)?;
        if !matches!(val_ty, Type::Uint(64)) {
            return Err(IrError::VerifyIntToPtrFromNonIntegerType(
                val_ty.as_string(self.context),
            ));
        }
        if ty.is_copy_type() {
            return Err(IrError::VerifyIntToPtrToCopyType(
                val_ty.as_string(self.context),
            ));
        }

        Ok(())
    }

    fn verify_load(&self, src_val: &Value) -> Result<(), IrError> {
        let src_ptr = self.get_pointer(src_val);
        if src_ptr.is_none() {
            if !self.is_ptr_argument(src_val) {
                Err(IrError::VerifyLoadFromNonPointer)
            } else {
                Ok(())
            }
        } else if !self.is_local_pointer(src_ptr.as_ref().unwrap()) {
            Err(IrError::VerifyLoadNonExistentPointer)
        } else {
            Ok(())
        }
    }

    fn verify_phi(&self, pairs: &[(Block, Value)]) -> Result<(), IrError> {
        if pairs.is_empty() {
            Ok(())
        } else if std::collections::HashSet::<&String>::from_iter(
            pairs
                .iter()
                .map(|(block, _)| &(self.context.blocks[block.0].label)),
        )
        .len()
            != pairs.len()
        {
            Err(IrError::VerifyPhiNonUniqueLabels)
        } else if pairs
            .iter()
            .map(|(_, v)| v.get_type(self.context))
            .reduce(|a, b| if self.opt_ty_not_eq(&a, &b) { None } else { b })
            .is_none()
        {
            Err(IrError::VerifyPhiInconsistentTypes)
        } else if let Some((from_block, _)) = pairs
            .iter()
            .find(|(from_block, _)| !self.cur_function.blocks.contains(from_block))
        {
            Err(IrError::VerifyPhiFromMissingBlock(
                self.context.blocks[from_block.0].label.clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_ret(
        &self,
        function: &FunctionContent,
        val: &Value,
        ty: &Type,
    ) -> Result<(), IrError> {
        if !function.return_type.eq(self.context, ty)
            || self.opt_ty_not_eq(&val.get_type(self.context), &Some(*ty))
        {
            Err(IrError::VerifyMismatchedReturnTypes(function.name.clone()))
        } else {
            Ok(())
        }
    }

    fn verify_state_load_store(
        &self,
        dst_val: &Value,
        val_type: &Type,
        key: &Value,
    ) -> Result<(), IrError> {
        if !matches!(self.get_pointer_type(dst_val), Some(ty) if ty.eq(self.context, val_type)) {
            Err(IrError::VerifyStateDestBadType(
                val_type.as_string(self.context),
            ))
        } else if !matches!(self.get_pointer_type(key), Some(Type::B256)) {
            Err(IrError::VerifyStateKeyBadType)
        } else {
            Ok(())
        }
    }

    fn verify_state_load_word(&self, key: &Value) -> Result<(), IrError> {
        if !matches!(self.get_pointer_type(key), Some(Type::B256)) {
            Err(IrError::VerifyStateKeyBadType)
        } else {
            Ok(())
        }
    }

    fn verify_state_store_word(&self, dst_val: &Value, key: &Value) -> Result<(), IrError> {
        if !matches!(self.get_pointer_type(key), Some(Type::B256)) {
            Err(IrError::VerifyStateKeyBadType)
        } else if !matches!(dst_val.get_type(self.context), Some(Type::Uint(64))) {
            Err(IrError::VerifyStateDestBadType(
                Type::Uint(64).as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_store(&self, dst_val: &Value, stored_val: &Value) -> Result<(), IrError> {
        let dst_ty = self.get_pointer_type(dst_val);
        if dst_ty.is_none() {
            Err(IrError::VerifyStoreToNonPointer)
        } else if self.opt_ty_not_eq(&dst_ty, &stored_val.get_type(self.context)) {
            Err(IrError::VerifyStoreMismatchedTypes)
        } else {
            match self.get_pointer(dst_val) {
                None => {
                    if !self.is_ptr_argument(dst_val) {
                        Err(IrError::VerifyStoreToNonPointer) // Should've been caught already.
                    } else {
                        Ok(())
                    }
                }
                Some(dst_ptr) => {
                    if !self.is_local_pointer(&dst_ptr) {
                        Err(IrError::VerifyStoreNonExistentPointer)
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    fn get_pointer(&self, ptr_val: &Value) -> Option<Pointer> {
        match &self.context.values[ptr_val.0].value {
            ValueDatum::Instruction(Instruction::GetPointer { base_ptr, .. }) => Some(*base_ptr),
            _otherwise => None,
        }
    }

    // This is a temporary workaround due to the fact that we don't support pointer arguments yet.
    // We do treat non-copy types as references anyways though so this is fine. Eventually, we
    // should allow function arguments to also be Pointer.
    //
    // Also, because we inline everything at the moment, this doesn't really matter and is added
    // simply to make the verifier happy.
    //
    fn is_ptr_argument(&self, ptr_val: &Value) -> bool {
        match &self.context.values[ptr_val.0].value {
            ValueDatum::Argument(arg_ty) => !arg_ty.is_copy_type(),
            _otherwise => false,
        }
    }

    fn get_pointer_type(&self, ptr_val: &Value) -> Option<Type> {
        match &self.context.values[ptr_val.0].value {
            ValueDatum::Instruction(Instruction::GetPointer { ptr_ty, .. }) => Some(*ptr_ty),
            ValueDatum::Argument(arg_ty) => match arg_ty.is_copy_type() {
                true => None,
                false => Some(*arg_ty),
            },
            _otherwise => None,
        }
    }

    fn is_local_pointer(&self, ptr: &Pointer) -> bool {
        self.cur_function.local_storage.values().any(|x| x == ptr)
    }

    // This is a really common operation above... calling `Value::get_type()` and then failing when
    // two don't match.
    fn opt_ty_not_eq(&self, l_ty: &Option<Type>, r_ty: &Option<Type>) -> bool {
        l_ty.is_none() || r_ty.is_none() || !l_ty.unwrap().eq(self.context, r_ty.as_ref().unwrap())
    }
}
