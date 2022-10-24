//! The base descriptor for various values within the IR.
//!
//! [`Value`]s can be function arguments, constants and instructions.  [`Instruction`]s generally
//! refer to each other and to constants via the [`Value`] wrapper.
//!
//! Like most IR data structures they are `Copy` and cheap to pass around by value.  They are
//! therefore also easy to replace, a common practice for optimization passes.

use crate::{
    constant::Constant,
    context::Context,
    instruction::Instruction,
    irtype::Type,
    metadata::{combine, MetadataIndex},
    pointer::Pointer,
    pretty::DebugWithContext,
    BlockArgument,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Value(#[in_context(values)] pub generational_arena::Index);

#[doc(hidden)]
#[derive(Debug, Clone, DebugWithContext)]
pub struct ValueContent {
    pub value:    ValueDatum,
    pub metadata: Option<MetadataIndex>,
}

#[doc(hidden)]
#[derive(Debug, Clone, DebugWithContext)]
pub enum ValueDatum {
    Argument(BlockArgument),
    Constant(Constant),
    Instruction(Instruction),
}

impl Value {
    /// Return a new argument [`Value`].
    pub fn new_argument(context: &mut Context, arg: BlockArgument) -> Value {
        let content = ValueContent {
            value:    ValueDatum::Argument(arg),
            metadata: None,
        };
        Value(context.values.insert(content))
    }

    /// Return a new constant [`Value`].
    pub fn new_constant(context: &mut Context, constant: Constant) -> Value {
        let content = ValueContent {
            value:    ValueDatum::Constant(constant),
            metadata: None,
        };
        Value(context.values.insert(content))
    }

    /// Return a new instruction [`Value`].
    pub fn new_instruction(context: &mut Context, instruction: Instruction) -> Value {
        let content = ValueContent {
            value:    ValueDatum::Instruction(instruction),
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
                    | Instruction::Revert(_)
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
                    | Instruction::Revert(..)
            ),
            ValueDatum::Argument(..) | ValueDatum::Constant(..) => false,
        }
    }

    /// If this value is an instruction and if any of its parameters is `old_val` then replace them
    /// with `new_val`.
    pub fn replace_instruction_value(&self, context: &mut Context, old_val: Value, new_val: Value) {
        if let ValueDatum::Instruction(instruction) =
            &mut context.values.get_mut(self.0).unwrap().value
        {
            instruction.replace_value(old_val, new_val);
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
    pub fn get_constant<'a>(&self, context: &'a Context) -> Option<&'a Constant> {
        if let ValueDatum::Constant(cn) = &context.values.get(self.0).unwrap().value {
            Some(cn)
        } else {
            None
        }
    }

    /// Iff this value is an argument, return its type.
    pub fn get_argument_type(&self, context: &Context) -> Option<Type> {
        if let ValueDatum::Argument(BlockArgument { ty, .. }) =
            &context.values.get(self.0).unwrap().value
        {
            Some(*ty)
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
            ValueDatum::Constant(c) => Some(c.ty),
            ValueDatum::Instruction(ins) => ins.get_type(context),
        }
    }

    /// Get the pointer argument for this value if there is one.  I.e., where get_ptr is
    /// essentially a Value wrapper around a Pointer, this function unwrap it.
    pub fn get_pointer(&self, context: &Context) -> Option<Pointer> {
        match &context.values[self.0].value {
            ValueDatum::Instruction(Instruction::GetPointer { base_ptr, .. }) => Some(*base_ptr),

            ValueDatum::Argument(BlockArgument {
                ty: Type::Pointer(ptr),
                ..
            }) => Some(*ptr),

            ValueDatum::Instruction(Instruction::InsertValue { aggregate, .. })
            | ValueDatum::Instruction(Instruction::ExtractValue { aggregate, .. }) => {
                aggregate.get_pointer(context)
            }

            _otherwise => None,
        }
    }

    /// Get the type for this value with any pointer stripped, if found.
    pub fn get_stripped_ptr_type(&self, context: &Context) -> Option<Type> {
        self.get_type(context).map(|f| f.strip_ptr_type(context))
    }
}
