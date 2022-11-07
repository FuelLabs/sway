use std::{
    collections::HashMap,
    fmt::{self, Write},
};

use derivative::Derivative;
use sway_types::{state::StateIndex, Ident, Span};

use crate::{
    declaration_engine::{de_get_enum, de_get_function, DeclMapping, DeclarationId, ReplaceDecls},
    language::{ty::*, *},
    type_system::*,
};

#[derive(Clone, Debug, Derivative)]
#[derivative(Eq)]
pub enum TyExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        call_path: CallPath,
        #[derivative(Eq(bound = ""))]
        contract_call_params: HashMap<String, TyExpression>,
        arguments: Vec<(Ident, TyExpression)>,
        function_decl_id: DeclarationId,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        self_state_idx: Option<StateIndex>,
        #[derivative(Eq(bound = ""))]
        selector: Option<ContractCallParams>,
    },
    LazyOperator {
        #[derivative(Eq(bound = ""))]
        op: LazyOp,
        lhs: Box<TyExpression>,
        rhs: Box<TyExpression>,
    },
    VariableExpression {
        name: Ident,
        span: Span,
        mutability: VariableMutability,
    },
    Tuple {
        fields: Vec<TyExpression>,
    },
    Array {
        contents: Vec<TyExpression>,
    },
    ArrayIndex {
        prefix: Box<TyExpression>,
        index: Box<TyExpression>,
    },
    StructExpression {
        struct_name: Ident,
        fields: Vec<TyStructExpressionField>,
        span: Span,
    },
    CodeBlock(TyCodeBlock),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TyExpression>,
        then: Box<TyExpression>,
        r#else: Option<Box<TyExpression>>,
    },
    AsmExpression {
        registers: Vec<TyAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span: Span,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TyExpression>,
        field_to_access: TyStructField,
        field_instantiation_span: Span,
        resolved_type_of_parent: TypeId,
    },
    TupleElemAccess {
        prefix: Box<TyExpression>,
        elem_to_access_num: usize,
        resolved_type_of_parent: TypeId,
        elem_to_access_span: Span,
    },
    EnumInstantiation {
        /// for printing
        enum_decl: DeclarationId,
        /// for printing
        variant_name: Ident,
        tag: usize,
        contents: Option<Box<TyExpression>>,
        /// If there is an error regarding this instantiation of the enum,
        /// use these spans as it points to the call site and not the declaration.
        /// They are also used in the language server.
        enum_instantiation_span: Span,
        variant_instantiation_span: Span,
    },
    AbiCast {
        abi_name: CallPath,
        address: Box<TyExpression>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span,
    },
    StorageAccess(TyStorageAccess),
    IntrinsicFunction(TyIntrinsicFunctionKind),
    /// a zero-sized type-system-only compile-time thing that is used for constructing ABI casts.
    AbiName(AbiName),
    /// grabs the enum tag from the particular enum and variant of the `exp`
    EnumTag {
        exp: Box<TyExpression>,
    },
    /// performs an unsafe cast from the `exp` to the type of the given enum `variant`
    UnsafeDowncast {
        exp: Box<TyExpression>,
        variant: TyEnumVariant,
    },
    WhileLoop {
        condition: Box<TyExpression>,
        body: TyCodeBlock,
    },
    Break,
    Continue,
    Reassignment(Box<TyReassignment>),
    StorageReassignment(Box<TyStorageReassignment>),
    Return(Box<TyExpression>),
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyExpressionVariant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (
                Self::FunctionApplication {
                    call_path: l_name,
                    arguments: l_arguments,
                    function_decl_id: l_function_decl_id,
                    ..
                },
                Self::FunctionApplication {
                    call_path: r_name,
                    arguments: r_arguments,
                    function_decl_id: r_function_decl_id,
                    ..
                },
            ) => {
                let l_function_decl =
                    de_get_function(l_function_decl_id.clone(), &Span::dummy()).unwrap();
                let r_function_decl =
                    de_get_function(r_function_decl_id.clone(), &Span::dummy()).unwrap();
                l_name == r_name
                    && l_arguments == r_arguments
                    && l_function_decl.body == r_function_decl.body
            }
            (
                Self::LazyOperator {
                    op: l_op,
                    lhs: l_lhs,
                    rhs: l_rhs,
                },
                Self::LazyOperator {
                    op: r_op,
                    lhs: r_lhs,
                    rhs: r_rhs,
                },
            ) => l_op == r_op && (**l_lhs) == (**r_lhs) && (**l_rhs) == (**r_rhs),
            (
                Self::VariableExpression {
                    name: l_name,
                    span: l_span,
                    mutability: l_mutability,
                },
                Self::VariableExpression {
                    name: r_name,
                    span: r_span,
                    mutability: r_mutability,
                },
            ) => l_name == r_name && l_span == r_span && l_mutability == r_mutability,
            (Self::Tuple { fields: l_fields }, Self::Tuple { fields: r_fields }) => {
                l_fields == r_fields
            }
            (
                Self::Array {
                    contents: l_contents,
                },
                Self::Array {
                    contents: r_contents,
                },
            ) => l_contents == r_contents,
            (
                Self::ArrayIndex {
                    prefix: l_prefix,
                    index: l_index,
                },
                Self::ArrayIndex {
                    prefix: r_prefix,
                    index: r_index,
                },
            ) => (**l_prefix) == (**r_prefix) && (**l_index) == (**r_index),
            (
                Self::StructExpression {
                    struct_name: l_struct_name,
                    fields: l_fields,
                    span: l_span,
                },
                Self::StructExpression {
                    struct_name: r_struct_name,
                    fields: r_fields,
                    span: r_span,
                },
            ) => {
                l_struct_name == r_struct_name
                    && l_fields.clone() == r_fields.clone()
                    && l_span == r_span
            }
            (Self::CodeBlock(l0), Self::CodeBlock(r0)) => l0 == r0,
            (
                Self::IfExp {
                    condition: l_condition,
                    then: l_then,
                    r#else: l_r,
                },
                Self::IfExp {
                    condition: r_condition,
                    then: r_then,
                    r#else: r_r,
                },
            ) => {
                (**l_condition) == (**r_condition)
                    && (**l_then) == (**r_then)
                    && if let (Some(l), Some(r)) = (l_r, r_r) {
                        (**l) == (**r)
                    } else {
                        true
                    }
            }
            (
                Self::AsmExpression {
                    registers: l_registers,
                    body: l_body,
                    returns: l_returns,
                    ..
                },
                Self::AsmExpression {
                    registers: r_registers,
                    body: r_body,
                    returns: r_returns,
                    ..
                },
            ) => {
                l_registers.clone() == r_registers.clone()
                    && l_body.clone() == r_body.clone()
                    && l_returns == r_returns
            }
            (
                Self::StructFieldAccess {
                    prefix: l_prefix,
                    field_to_access: l_field_to_access,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                Self::StructFieldAccess {
                    prefix: r_prefix,
                    field_to_access: r_field_to_access,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix) == (**r_prefix)
                    && l_field_to_access == r_field_to_access
                    && look_up_type_id(*l_resolved_type_of_parent)
                        == look_up_type_id(*r_resolved_type_of_parent)
            }
            (
                Self::TupleElemAccess {
                    prefix: l_prefix,
                    elem_to_access_num: l_elem_to_access_num,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                Self::TupleElemAccess {
                    prefix: r_prefix,
                    elem_to_access_num: r_elem_to_access_num,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix) == (**r_prefix)
                    && l_elem_to_access_num == r_elem_to_access_num
                    && look_up_type_id(*l_resolved_type_of_parent)
                        == look_up_type_id(*r_resolved_type_of_parent)
            }
            (
                Self::EnumInstantiation {
                    enum_decl: l_enum_decl,
                    variant_name: l_variant_name,
                    tag: l_tag,
                    contents: l_contents,
                    ..
                },
                Self::EnumInstantiation {
                    enum_decl: r_enum_decl,
                    variant_name: r_variant_name,
                    tag: r_tag,
                    contents: r_contents,
                    ..
                },
            ) => {
                l_enum_decl == r_enum_decl
                    && l_variant_name == r_variant_name
                    && l_tag == r_tag
                    && if let (Some(l_contents), Some(r_contents)) = (l_contents, r_contents) {
                        (**l_contents) == (**r_contents)
                    } else {
                        true
                    }
            }
            (
                Self::AbiCast {
                    abi_name: l_abi_name,
                    address: l_address,
                    ..
                },
                Self::AbiCast {
                    abi_name: r_abi_name,
                    address: r_address,
                    ..
                },
            ) => l_abi_name == r_abi_name && (**l_address) == (**r_address),
            (Self::IntrinsicFunction(l_kind), Self::IntrinsicFunction(r_kind)) => l_kind == r_kind,
            (
                Self::UnsafeDowncast {
                    exp: l_exp,
                    variant: l_variant,
                },
                Self::UnsafeDowncast {
                    exp: r_exp,
                    variant: r_variant,
                },
            ) => *l_exp == *r_exp && l_variant == r_variant,
            (Self::EnumTag { exp: l_exp }, Self::EnumTag { exp: r_exp }) => *l_exp == *r_exp,
            (Self::StorageAccess(l_exp), Self::StorageAccess(r_exp)) => *l_exp == *r_exp,
            (
                Self::WhileLoop {
                    body: l_body,
                    condition: l_condition,
                },
                Self::WhileLoop {
                    body: r_body,
                    condition: r_condition,
                },
            ) => *l_body == *r_body && l_condition == r_condition,
            _ => false,
        }
    }
}

impl CopyTypes for TyExpressionVariant {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                ref mut function_decl_id,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.copy_types(type_mapping));
                let new_decl_id = function_decl_id
                    .clone()
                    .copy_types_and_insert_new(type_mapping);
                function_decl_id.replace_id(*new_decl_id);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).copy_types(type_mapping);
                (*rhs).copy_types(type_mapping);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields.iter_mut().for_each(|x| x.copy_types(type_mapping)),
            Array { contents } => contents.iter_mut().for_each(|x| x.copy_types(type_mapping)),
            ArrayIndex { prefix, index } => {
                (*prefix).copy_types(type_mapping);
                (*index).copy_types(type_mapping);
            }
            StructExpression { fields, .. } => {
                fields.iter_mut().for_each(|x| x.copy_types(type_mapping))
            }
            CodeBlock(block) => {
                block.copy_types(type_mapping);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.copy_types(type_mapping);
                then.copy_types(type_mapping);
                if let Some(ref mut r#else) = r#else {
                    r#else.copy_types(type_mapping);
                }
            }
            AsmExpression {
                registers, //: Vec<TyAsmRegisterDeclaration>,
                ..
            } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.copy_types(type_mapping));
            }
            // like a variable expression but it has multiple parts,
            // like looking up a field in a struct
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.copy_types(type_mapping);
                field_to_access.copy_types(type_mapping);
                prefix.copy_types(type_mapping);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.copy_types(type_mapping);
                prefix.copy_types(type_mapping);
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                enum_decl.copy_types(type_mapping);
                if let Some(ref mut contents) = contents {
                    contents.copy_types(type_mapping)
                };
            }
            AbiCast { address, .. } => address.copy_types(type_mapping),
            // storage is never generic and cannot be monomorphized
            StorageAccess { .. } => (),
            IntrinsicFunction(kind) => {
                kind.copy_types(type_mapping);
            }
            EnumTag { exp } => {
                exp.copy_types(type_mapping);
            }
            UnsafeDowncast { exp, variant } => {
                exp.copy_types(type_mapping);
                variant.copy_types(type_mapping);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.copy_types(type_mapping);
                body.copy_types(type_mapping);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => reassignment.copy_types(type_mapping),
            StorageReassignment(..) => (),
            Return(stmt) => stmt.copy_types(type_mapping),
        }
    }
}

impl ReplaceSelfType for TyExpressionVariant {
    fn replace_self_type(&mut self, self_type: TypeId) {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                ref mut function_decl_id,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.replace_self_type(self_type));
                let new_decl_id = function_decl_id
                    .clone()
                    .replace_self_type_and_insert_new(self_type);
                function_decl_id.replace_id(*new_decl_id);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).replace_self_type(self_type);
                (*rhs).replace_self_type(self_type);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields
                .iter_mut()
                .for_each(|x| x.replace_self_type(self_type)),
            Array { contents } => contents
                .iter_mut()
                .for_each(|x| x.replace_self_type(self_type)),
            ArrayIndex { prefix, index } => {
                (*prefix).replace_self_type(self_type);
                (*index).replace_self_type(self_type);
            }
            StructExpression { fields, .. } => fields
                .iter_mut()
                .for_each(|x| x.replace_self_type(self_type)),
            CodeBlock(block) => {
                block.replace_self_type(self_type);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.replace_self_type(self_type);
                then.replace_self_type(self_type);
                if let Some(ref mut r#else) = r#else {
                    r#else.replace_self_type(self_type);
                }
            }
            AsmExpression { registers, .. } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.replace_self_type(self_type));
            }
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.replace_self_type(self_type);
                field_to_access.replace_self_type(self_type);
                prefix.replace_self_type(self_type);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.replace_self_type(self_type);
                prefix.replace_self_type(self_type);
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                enum_decl.replace_self_type(self_type);
                if let Some(ref mut contents) = contents {
                    contents.replace_self_type(self_type)
                };
            }
            AbiCast { address, .. } => address.replace_self_type(self_type),
            StorageAccess { .. } => (),
            IntrinsicFunction(kind) => {
                kind.replace_self_type(self_type);
            }
            EnumTag { exp } => {
                exp.replace_self_type(self_type);
            }
            UnsafeDowncast { exp, variant } => {
                exp.replace_self_type(self_type);
                variant.replace_self_type(self_type);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.replace_self_type(self_type);
                body.replace_self_type(self_type);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => reassignment.replace_self_type(self_type),
            StorageReassignment(..) => (),
            Return(stmt) => stmt.replace_self_type(self_type),
        }
    }
}

impl ReplaceDecls for TyExpressionVariant {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping) {
        use TyExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                ref mut function_decl_id,
                ref mut arguments,
                ..
            } => {
                function_decl_id.replace_decls(decl_mapping);
                let new_decl_id = function_decl_id
                    .clone()
                    .replace_decls_and_insert_new(decl_mapping);
                function_decl_id.replace_id(*new_decl_id);
                for (_, arg) in arguments.iter_mut() {
                    arg.replace_decls(decl_mapping);
                }
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).replace_decls(decl_mapping);
                (*rhs).replace_decls(decl_mapping);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields
                .iter_mut()
                .for_each(|x| x.replace_decls(decl_mapping)),
            Array { contents } => contents
                .iter_mut()
                .for_each(|x| x.replace_decls(decl_mapping)),
            ArrayIndex { prefix, index } => {
                (*prefix).replace_decls(decl_mapping);
                (*index).replace_decls(decl_mapping);
            }
            StructExpression { fields, .. } => fields
                .iter_mut()
                .for_each(|x| x.replace_decls(decl_mapping)),
            CodeBlock(block) => {
                block.replace_decls(decl_mapping);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.replace_decls(decl_mapping);
                then.replace_decls(decl_mapping);
                if let Some(ref mut r#else) = r#else {
                    r#else.replace_decls(decl_mapping);
                }
            }
            AsmExpression { .. } => {}
            StructFieldAccess { prefix, .. } => {
                prefix.replace_decls(decl_mapping);
            }
            TupleElemAccess { prefix, .. } => {
                prefix.replace_decls(decl_mapping);
            }
            EnumInstantiation {
                enum_decl: _,
                contents,
                ..
            } => {
                // TODO: replace enum decl
                //enum_decl.replace_decls(decl_mapping);
                if let Some(ref mut contents) = contents {
                    contents.replace_decls(decl_mapping);
                };
            }
            AbiCast { address, .. } => address.replace_decls(decl_mapping),
            StorageAccess { .. } => (),
            IntrinsicFunction(_) => {}
            EnumTag { exp } => {
                exp.replace_decls(decl_mapping);
            }
            UnsafeDowncast { exp, .. } => {
                exp.replace_decls(decl_mapping);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.replace_decls(decl_mapping);
                body.replace_decls(decl_mapping);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => reassignment.replace_decls(decl_mapping),
            StorageReassignment(..) => (),
            Return(stmt) => stmt.replace_decls(decl_mapping),
        }
    }
}

impl fmt::Display for TyExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TyExpressionVariant::Literal(lit) => format!("literal {}", lit),
            TyExpressionVariant::FunctionApplication {
                call_path: name, ..
            } => {
                format!("\"{}\" fn entry", name.suffix.as_str())
            }
            TyExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TyExpressionVariant::Tuple { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple({})", fields)
            }
            TyExpressionVariant::Array { .. } => "array".into(),
            TyExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TyExpressionVariant::StructExpression { struct_name, .. } => {
                format!("\"{}\" struct init", struct_name.as_str())
            }
            TyExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TyExpressionVariant::FunctionParameter => "fn param access".into(),
            TyExpressionVariant::IfExp { .. } => "if exp".into(),
            TyExpressionVariant::AsmExpression { .. } => "inline asm".into(),
            TyExpressionVariant::AbiCast { abi_name, .. } => {
                format!("abi cast {}", abi_name.suffix.as_str())
            }
            TyExpressionVariant::StructFieldAccess {
                resolved_type_of_parent,
                field_to_access,
                ..
            } => {
                format!(
                    "\"{}.{}\" struct field access",
                    look_up_type_id(*resolved_type_of_parent),
                    field_to_access.name
                )
            }
            TyExpressionVariant::TupleElemAccess {
                resolved_type_of_parent,
                elem_to_access_num,
                ..
            } => {
                format!(
                    "\"{}.{}\" tuple index",
                    look_up_type_id(*resolved_type_of_parent),
                    elem_to_access_num
                )
            }
            TyExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.as_str())
            }
            TyExpressionVariant::EnumInstantiation {
                tag,
                enum_decl,
                variant_name,
                enum_instantiation_span,
                ..
            } => {
                let enum_decl = de_get_enum(enum_decl.clone(), enum_instantiation_span).unwrap();
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_decl.name.as_str(),
                    variant_name.as_str(),
                    tag
                )
            }
            TyExpressionVariant::StorageAccess(access) => {
                format!("storage field {} access", access.storage_field_name())
            }
            TyExpressionVariant::IntrinsicFunction(kind) => kind.to_string(),
            TyExpressionVariant::AbiName(n) => format!("ABI name {}", n),
            TyExpressionVariant::EnumTag { exp } => {
                format!("({} as tag)", look_up_type_id(exp.return_type))
            }
            TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                format!("({} as {})", look_up_type_id(exp.return_type), variant.name)
            }
            TyExpressionVariant::WhileLoop { condition, .. } => {
                format!("while loop on {}", condition)
            }
            TyExpressionVariant::Break => "break".to_string(),
            TyExpressionVariant::Continue => "continue".to_string(),
            TyExpressionVariant::Reassignment(reassignment) => {
                let mut place = reassignment.lhs_base_name.to_string();
                for index in &reassignment.lhs_indices {
                    place.push('.');
                    match index {
                        ProjectionKind::StructField { name } => place.push_str(name.as_str()),
                        ProjectionKind::TupleField { index, .. } => {
                            write!(&mut place, "{}", index).unwrap();
                        }
                    }
                }
                format!("reassignment to {}", place)
            }
            TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                let place: String = {
                    storage_reassignment
                        .fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect()
                };
                format!("storage reassignment to {}", place)
            }
            TyExpressionVariant::Return(exp) => {
                format!("return {}", *exp)
            }
        };
        write!(f, "{}", s)
    }
}

impl TyExpressionVariant {
    /// Returns `self` as a literal, if possible.
    pub(crate) fn extract_literal_value(&self) -> Option<Literal> {
        match self {
            TyExpressionVariant::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TyExpression> {
        match self {
            TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                let mut buf = condition.gather_return_statements();
                buf.append(&mut then.gather_return_statements());
                if let Some(ref r#else) = r#else {
                    buf.append(&mut r#else.gather_return_statements());
                }
                buf
            }
            TyExpressionVariant::CodeBlock(TyCodeBlock { contents, .. }) => {
                let mut buf = vec![];
                for node in contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            TyExpressionVariant::WhileLoop { condition, body } => {
                let mut buf = condition.gather_return_statements();
                for node in &body.contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            TyExpressionVariant::Reassignment(reassignment) => {
                reassignment.rhs.gather_return_statements()
            }
            TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                storage_reassignment.rhs.gather_return_statements()
            }
            TyExpressionVariant::LazyOperator { lhs, rhs, .. } => [lhs, rhs]
                .into_iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::Tuple { fields } => fields
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::Array { contents } => contents
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::ArrayIndex { prefix, index } => [prefix, index]
                .into_iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::StructFieldAccess { prefix, .. } => {
                prefix.gather_return_statements()
            }
            TyExpressionVariant::TupleElemAccess { prefix, .. } => {
                prefix.gather_return_statements()
            }
            TyExpressionVariant::EnumInstantiation { contents, .. } => contents
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::AbiCast { address, .. } => address.gather_return_statements(),
            TyExpressionVariant::IntrinsicFunction(intrinsic_function_kind) => {
                intrinsic_function_kind
                    .arguments
                    .iter()
                    .flat_map(|expr| expr.gather_return_statements())
                    .collect()
            }
            TyExpressionVariant::StructExpression { fields, .. } => fields
                .iter()
                .flat_map(|field| field.value.gather_return_statements())
                .collect(),
            TyExpressionVariant::FunctionApplication {
                contract_call_params,
                arguments,
                selector,
                ..
            } => contract_call_params
                .values()
                .chain(arguments.iter().map(|(_name, expr)| expr))
                .chain(
                    selector
                        .iter()
                        .map(|contract_call_params| &*contract_call_params.contract_address),
                )
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TyExpressionVariant::EnumTag { exp } => exp.gather_return_statements(),
            TyExpressionVariant::UnsafeDowncast { exp, .. } => exp.gather_return_statements(),

            TyExpressionVariant::Return(exp) => {
                vec![exp]
            }
            // if it is impossible for an expression to contain a return _statement_ (not an
            // implicit return!), put it in the pattern below.
            TyExpressionVariant::Literal(_)
            | TyExpressionVariant::FunctionParameter { .. }
            | TyExpressionVariant::AsmExpression { .. }
            | TyExpressionVariant::VariableExpression { .. }
            | TyExpressionVariant::AbiName(_)
            | TyExpressionVariant::StorageAccess { .. }
            | TyExpressionVariant::Break
            | TyExpressionVariant::Continue => vec![],
        }
    }
}
