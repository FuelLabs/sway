//! The base descriptor for various values within the IR.
//!
//! [`Value`]s can be function arguments, constants and instructions.  [`Instruction`]s generally
//! refer to each other and to constants via the [`Value`] wrapper.
//!
//! Like most IR data structures they are `Copy` and cheap to pass around by value.  They are
//! therefore also easy to replace, a common practise for optimization passes.

use sway_types::span::Span;

use crate::{
    constant::Constant, context::Context, instruction::Instruction, irtype::Type,
    metadata::MetadataIndex,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Value(pub generational_arena::Index);

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct ValueContent {
    pub value: ValueDatum,
    pub span_md_idx: Option<MetadataIndex>,
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum ValueDatum {
    Argument(Type),
    Constant(Constant),
    Instruction(Instruction),
}

impl Value {
    /// Return a new argument [`Value`].
    pub fn new_argument(
        context: &mut Context,
        ty: Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let content = ValueContent {
            value: ValueDatum::Argument(ty),
            span_md_idx,
        };
        Value(context.values.insert(content))
    }

    /// Return a new constant [`Value`].
    pub fn new_constant(
        context: &mut Context,
        constant: Constant,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let content = ValueContent {
            value: ValueDatum::Constant(constant),
            span_md_idx,
        };
        Value(context.values.insert(content))
    }

    /// Return a new instruction [`Value`].
    pub fn new_instruction(
        context: &mut Context,
        instruction: Instruction,
        opt_span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let content = ValueContent {
            value: ValueDatum::Instruction(instruction),
            span_md_idx: opt_span_md_idx,
        };
        Value(context.values.insert(content))
    }

    /// Return this value's source span.
    pub fn get_span(&self, context: &Context) -> Option<Span> {
        // We unwrap the Result for now, until we refactor Span to not need a source string, in
        // which case there will be no need to open and read a file, and no Result involved.
        context.values[self.0]
            .span_md_idx
            .map(|idx| idx.to_span(context))
            .transpose()
            .expect("A valid span.")
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
            ),
            _ => false,
        }
    }

    //pub fn get_constant(&self, context: &Context) -> Constant {
    //    if let ValueContent::Constant(c) = &context.values[self.0] {
    //        c.clone()
    //    } else {
    //        panic!("Value is not a constant.")
    //    }
    //}

    /// If this value is an instruction and if any of its parameters is `old_val` then replace them
    /// with `new_val`.
    pub fn replace_instruction_value(&self, context: &mut Context, old_val: Value, new_val: Value) {
        if let ValueDatum::Instruction(instruction) =
            &mut context.values.get_mut(self.0).unwrap().value
        {
            instruction.replace_value(old_val, new_val);
        }
    }

    /// Get the type for this value, if found.
    ///
    /// Arguments and constants always have a type, but only some instructions do.
    pub fn get_type(&self, context: &Context) -> Option<Type> {
        match &context.values[self.0].value {
            ValueDatum::Argument(ty) => Some(*ty),
            ValueDatum::Constant(c) => Some(c.ty),
            ValueDatum::Instruction(ins) => ins.get_type(context),
        }
    }

    //pub fn is_bool_ty(&self, context: &Context) -> bool {
    //    self.get_type(context)
    //        .map(|ty| ty == Type::Bool)
    //        .unwrap_or(false)
    //}
}
