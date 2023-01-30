//! The base descriptor for various values within the IR.
//!
//! [`Value`]s can be function arguments, constants and instructions.  [`Instruction`]s generally
//! refer to each other and to constants via the [`Value`] wrapper.
//!
//! Like most IR data structures they are `Copy` and cheap to pass around by value.  They are
//! therefore also easy to replace, a common practice for optimization passes.

use rustc_hash::FxHashMap;

use crate::{
    constant::Constant,
    context::Context,
    instruction::Instruction,
    irtype::Type,
    metadata::{combine, MetadataIndex},
    pretty::DebugWithContext,
    BlockArgument, FuelVmInstruction,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Value(#[in_context(values)] pub generational_arena::Index);

#[doc(hidden)]
#[derive(Debug, Clone, DebugWithContext)]
pub struct ValueContent {
    pub value: ValueDatum,
    pub metadata: Option<MetadataIndex>,
}

#[doc(hidden)]
#[derive(Debug, Clone, DebugWithContext)]
pub enum ValueDatum {
    Argument(BlockArgument),
    Configurable(Constant),
    Constant(Constant),
    Instruction(Instruction),
}

impl Value {
    /// Return a new argument [`Value`].
    pub fn new_argument(context: &mut Context, arg: BlockArgument) -> Value {
        let content = ValueContent {
            value: ValueDatum::Argument(arg),
            metadata: None,
        };
        Value(context.values.insert(content))
    }

    /// Return a new constant [`Value`].
    pub fn new_configurable(context: &mut Context, constant: Constant) -> Value {
        let content = ValueContent {
            value: ValueDatum::Configurable(constant),
            metadata: None,
        };
        Value(context.values.insert(content))
    }

    /// Return a new constant [`Value`].
    pub fn new_constant(context: &mut Context, constant: Constant) -> Value {
        let content = ValueContent {
            value: ValueDatum::Constant(constant),
            metadata: None,
        };
        Value(context.values.insert(content))
    }

    /// Return a new instruction [`Value`].
    pub fn new_instruction(context: &mut Context, instruction: Instruction) -> Value {
        let content = ValueContent {
            value: ValueDatum::Instruction(instruction),
            metadata: None,
        };
        Value(context.values.insert(content))
    }

    /// Add some metadata to this value.
    ///
    /// As a convenience the `md_idx` argument is an `Option`, in which case this function is a
    /// no-op.
    ///
    /// If there is no existing metadata then the new metadata are added alone. Otherwise the new
    /// metadatum are added to the list of metadata.
    pub fn add_metadatum(self, context: &mut Context, md_idx: Option<MetadataIndex>) -> Self {
        if md_idx.is_some() {
            let orig_md = context.values[self.0].metadata;
            let new_md = combine(context, &orig_md, &md_idx);
            context.values[self.0].metadata = new_md;
        }
        self
    }

    /// Return this value's metadata.
    pub fn get_metadata(&self, context: &Context) -> Option<MetadataIndex> {
        context.values[self.0].metadata
    }

    /// Return whether this is a constant value.
    pub fn is_configurable(&self, context: &Context) -> bool {
        matches!(context.values[self.0].value, ValueDatum::Configurable(_))
    }

    /// Return whether this is a constant value.
    pub fn is_constant(&self, context: &Context) -> bool {
        matches!(context.values[self.0].value, ValueDatum::Constant(_))
    }

    /// Return whether this value is an instruction, and specifically a 'terminator'.
    ///
    /// A terminator is always the last instruction in a block (and may not appear anywhere else)
    /// and is either a branch or return.
    pub fn is_terminator(&self, context: &Context) -> bool {
        match &context.values[self.0].value {
            ValueDatum::Instruction(ins) => matches!(
                ins,
                Instruction::Branch(_)
                    | Instruction::ConditionalBranch { .. }
                    | Instruction::Ret(_, _)
                    | Instruction::FuelVm(FuelVmInstruction::Revert(_))
            ),
            _ => false,
        }
    }

    pub fn is_diverging(&self, context: &Context) -> bool {
        match &context.values[self.0].value {
            ValueDatum::Instruction(ins) => matches!(
                ins,
                Instruction::Branch(..)
                    | Instruction::ConditionalBranch { .. }
                    | Instruction::Ret(..)
                    | Instruction::FuelVm(FuelVmInstruction::Revert(..))
            ),
            ValueDatum::Argument(..) | ValueDatum::Configurable(..) | ValueDatum::Constant(..) => {
                false
            }
        }
    }

    /// If this value is an instruction and if any of its parameters is `old_val` then replace them
    /// with `new_val`.
    pub fn replace_instruction_values(
        &self,
        context: &mut Context,
        replace_map: &FxHashMap<Value, Value>,
    ) {
        if let ValueDatum::Instruction(instruction) =
            &mut context.values.get_mut(self.0).unwrap().value
        {
            instruction.replace_values(replace_map);
        }
    }

    /// Replace this value with another one, in-place.
    pub fn replace(&self, context: &mut Context, other: ValueDatum) {
        context.values[self.0].value = other;
    }

    /// Get a reference to this value as an instruction, iff it is one.
    pub fn get_instruction<'a>(&self, context: &'a Context) -> Option<&'a Instruction> {
        if let ValueDatum::Instruction(instruction) = &context.values.get(self.0).unwrap().value {
            Some(instruction)
        } else {
            None
        }
    }

    /// Get a mutable reference to this value as an instruction, iff it is one.
    pub fn get_instruction_mut<'a>(&self, context: &'a mut Context) -> Option<&'a mut Instruction> {
        if let ValueDatum::Instruction(instruction) =
            &mut context.values.get_mut(self.0).unwrap().value
        {
            Some(instruction)
        } else {
            None
        }
    }

    /// Get a reference to this value as a constant, iff it is one.
    pub fn get_configurable<'a>(&self, context: &'a Context) -> Option<&'a Constant> {
        if let ValueDatum::Configurable(cn) = &context.values.get(self.0).unwrap().value {
            Some(cn)
        } else {
            None
        }
    }

    /// Get a reference to this value as a constant, iff it is one.
    pub fn get_constant<'a>(&self, context: &'a Context) -> Option<&'a Constant> {
        if let ValueDatum::Constant(cn) = &context.values.get(self.0).unwrap().value {
            Some(cn)
        } else {
            None
        }
    }

    /// Iff this value is an argument, return its type.
    pub fn get_argument_type_and_byref(&self, context: &Context) -> Option<(Type, bool)> {
        if let ValueDatum::Argument(BlockArgument { ty, by_ref, .. }) =
            &context.values.get(self.0).unwrap().value
        {
            Some((*ty, *by_ref))
        } else {
            None
        }
    }

    /// Get the type for this value, if found.
    ///
    /// Arguments and constants always have a type, but only some instructions do.
    pub fn get_type(&self, context: &Context) -> Option<Type> {
        match &context.values[self.0].value {
            ValueDatum::Argument(BlockArgument { ty, .. }) => Some(*ty),
            ValueDatum::Configurable(c) => Some(c.ty),
            ValueDatum::Constant(c) => Some(c.ty),
            ValueDatum::Instruction(ins) => ins.get_type(context),
        }
    }
}
