use super::*;

use crate::{parse_tree::AsmOp, semantic_analysis::ast_node::*, Ident};
use std::collections::HashMap;
use sway_types::state::StateIndex;

use derivative::Derivative;

#[derive(Clone, Debug)]
pub(crate) struct ContractCallMetadata {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TypedExpression>,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Eq)]
pub(crate) enum TypedExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        name: CallPath,
        #[derivative(Eq(bound = ""))]
        contract_call_params: HashMap<String, TypedExpression>,
        arguments: Vec<(Ident, TypedExpression)>,
        function_body: TypedCodeBlock,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        #[derivative(Eq(bound = ""))]
        selector: Option<ContractCallMetadata>,
    },
    LazyOperator {
        #[derivative(Eq(bound = ""))]
        op: LazyOp,
        lhs: Box<TypedExpression>,
        rhs: Box<TypedExpression>,
    },
    VariableExpression {
        name: Ident,
    },
    Tuple {
        fields: Vec<TypedExpression>,
    },
    Array {
        contents: Vec<TypedExpression>,
    },
    ArrayIndex {
        prefix: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },
    StructExpression {
        struct_name: Ident,
        fields: Vec<TypedStructExpressionField>,
    },
    CodeBlock(TypedCodeBlock),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TypedExpression>,
        then: Box<TypedExpression>,
        r#else: Option<Box<TypedExpression>>,
    },
    AsmExpression {
        registers: Vec<TypedAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span: Span,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TypedExpression>,
        field_to_access: TypedStructField,
        resolved_type_of_parent: TypeId,
    },
    IfLet {
        enum_type: TypeId,
        expr: Box<TypedExpression>,
        variant: TypedEnumVariant,
        variable_to_assign: Ident,
        then: TypedCodeBlock,
        r#else: Option<Box<TypedExpression>>,
    },
    TupleElemAccess {
        prefix: Box<TypedExpression>,
        elem_to_access_num: usize,
        resolved_type_of_parent: TypeId,
        elem_to_access_span: Span,
    },
    EnumInstantiation {
        /// for printing
        enum_decl: TypedEnumDeclaration,
        /// for printing
        variant_name: Ident,
        tag: usize,
        contents: Option<Box<TypedExpression>>,
        /// If there is an error regarding this instantiation of the enum,
        /// use this span as it points to the call site and not the declaration.
        instantiation_span: Span,
    },
    AbiCast {
        abi_name: CallPath,
        address: Box<TypedExpression>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span,
    },
    #[allow(dead_code)]
    StorageAccess(TypeCheckedStorageAccess),
    TypeProperty {
        property: BuiltinProperty,
        type_id: TypeId,
        span: Span,
    },
    SizeOfValue {
        expr: Box<TypedExpression>,
    },
    /// a zero-sized type-system-only compile-time thing that is used for constructing ABI casts.
    AbiName(AbiName),
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedExpressionVariant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (
                Self::FunctionApplication {
                    name: l_name,
                    arguments: l_arguments,
                    function_body: l_function_body,
                    ..
                },
                Self::FunctionApplication {
                    name: r_name,
                    arguments: r_arguments,
                    function_body: r_function_body,
                    ..
                },
            ) => {
                l_name == r_name && l_arguments == r_arguments && l_function_body == r_function_body
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
                Self::VariableExpression { name: l_name },
                Self::VariableExpression { name: r_name },
            ) => l_name == r_name,
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
                },
                Self::StructExpression {
                    struct_name: r_struct_name,
                    fields: r_fields,
                },
            ) => l_struct_name == r_struct_name && l_fields.clone() == r_fields.clone(),
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
                },
                Self::StructFieldAccess {
                    prefix: r_prefix,
                    field_to_access: r_field_to_access,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                },
            ) => {
                (**l_prefix) == (**r_prefix)
                    && l_field_to_access == r_field_to_access
                    && look_up_type_id(*l_resolved_type_of_parent)
                        == look_up_type_id(*r_resolved_type_of_parent)
            }
            (
                Self::IfLet {
                    enum_type: l_enum_type,
                    expr: l_expr,
                    variant: l_variant,
                    variable_to_assign: l_variable_to_assign,
                    then: l_then,
                    r#else: l_r,
                },
                Self::IfLet {
                    enum_type: r_enum_type,
                    expr: r_expr,
                    variant: r_variant,
                    variable_to_assign: r_variable_to_assign,
                    then: r_then,
                    r#else: r_r,
                },
            ) => {
                look_up_type_id(*l_enum_type) == look_up_type_id(*r_enum_type)
                    && (**l_expr) == (**r_expr)
                    && l_variant == r_variant
                    && l_variable_to_assign == r_variable_to_assign
                    && l_then == r_then
                    && if let (Some(l_r), Some(r_r)) = (l_r, r_r) {
                        (**l_r) == (**r_r)
                    } else {
                        true
                    }
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
            (
                Self::TypeProperty {
                    property: l_prop,
                    type_id: l_type_id,
                    ..
                },
                Self::TypeProperty {
                    property: r_prop,
                    type_id: r_type_id,
                    ..
                },
            ) => l_prop == r_prop && look_up_type_id(*l_type_id) == look_up_type_id(*r_type_id),
            (Self::SizeOfValue { expr: l_expr }, Self::SizeOfValue { expr: r_expr }) => {
                l_expr == r_expr
            }
            _ => false,
        }
    }
}

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug)]
pub struct TypeCheckedStorageAccess {
    pub(crate) fields: Vec<TypeCheckedStorageAccessDescriptor>,
    pub(crate) ix: StateIndex,
}

impl TypeCheckedStorageAccess {
    pub fn storage_field_name(&self) -> Ident {
        self.fields[0].name.clone()
    }
    pub fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
}

/// Describes a single subfield access in the sequence when accessing a subfield within storage.
#[derive(Clone, Debug)]
pub struct TypeCheckedStorageAccessDescriptor {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAsmRegisterDeclaration {
    pub(crate) initializer: Option<TypedExpression>,
    pub(crate) name: Ident,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedAsmRegisterDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && if let (Some(l), Some(r)) = (self.initializer.clone(), other.initializer.clone()) {
                l == r
            } else {
                true
            }
    }
}

impl TypedAsmRegisterDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping)
        }
    }
}

impl TypedExpressionVariant {
    pub(crate) fn pretty_print(&self) -> String {
        match self {
            TypedExpressionVariant::Literal(lit) => format!(
                "literal {}",
                match lit {
                    Literal::U8(content) => content.to_string(),
                    Literal::U16(content) => content.to_string(),
                    Literal::U32(content) => content.to_string(),
                    Literal::U64(content) => content.to_string(),
                    Literal::Numeric(content) => content.to_string(),
                    Literal::String(content) => content.as_str().to_string(),
                    Literal::Boolean(content) => content.to_string(),
                    Literal::Byte(content) => content.to_string(),
                    Literal::B256(content) => content
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                }
            ),
            TypedExpressionVariant::FunctionApplication { name, .. } => {
                format!("\"{}\" fn entry", name.suffix.as_str())
            }
            TypedExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TypedExpressionVariant::Tuple { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| field.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple({})", fields)
            }
            TypedExpressionVariant::Array { .. } => "array".into(),
            TypedExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TypedExpressionVariant::StructExpression { struct_name, .. } => {
                format!("\"{}\" struct init", struct_name.as_str())
            }
            TypedExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TypedExpressionVariant::FunctionParameter => "fn param access".into(),
            TypedExpressionVariant::IfExp { .. } => "if exp".into(),
            TypedExpressionVariant::AsmExpression { .. } => "inline asm".into(),
            TypedExpressionVariant::AbiCast { abi_name, .. } => {
                format!("abi cast {}", abi_name.suffix.as_str())
            }
            TypedExpressionVariant::StructFieldAccess {
                resolved_type_of_parent,
                field_to_access,
                ..
            } => {
                format!(
                    "\"{}.{}\" struct field access",
                    look_up_type_id(*resolved_type_of_parent).friendly_type_str(),
                    field_to_access.name
                )
            }
            TypedExpressionVariant::IfLet {
                enum_type, variant, ..
            } => {
                format!(
                    "if let {}::{}",
                    enum_type.friendly_type_str(),
                    variant.name.as_str()
                )
            }
            TypedExpressionVariant::TupleElemAccess {
                resolved_type_of_parent,
                elem_to_access_num,
                ..
            } => {
                format!(
                    "\"{}.{}\" tuple index",
                    look_up_type_id(*resolved_type_of_parent).friendly_type_str(),
                    elem_to_access_num
                )
            }
            TypedExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.as_str())
            }
            TypedExpressionVariant::EnumInstantiation {
                tag,
                enum_decl,
                variant_name,
                ..
            } => {
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_decl.name.as_str(),
                    variant_name.as_str(),
                    tag
                )
            }
            TypedExpressionVariant::StorageAccess(access) => {
                format!("storage field {} access", access.storage_field_name())
            }
            TypedExpressionVariant::TypeProperty {
                property, type_id, ..
            } => {
                let type_str = look_up_type_id(*type_id).friendly_type_str();
                match property {
                    BuiltinProperty::SizeOfType => format!("size_of({type_str:?})"),
                    BuiltinProperty::IsRefType => format!("is_ref_type({type_str:?})"),
                }
            }
            TypedExpressionVariant::SizeOfValue { expr } => {
                format!("size_of_val({:?})", expr.pretty_print())
            }
            TypedExpressionVariant::AbiName(n) => format!("ABI name {}", n),
        }
    }

    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        use TypedExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                function_body,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.copy_types(type_mapping));
                function_body.copy_types(type_mapping);
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
                registers, //: Vec<TypedAsmRegisterDeclaration>,
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
                *resolved_type_of_parent = if let Some(matching_id) =
                    look_up_type_id(*resolved_type_of_parent).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*resolved_type_of_parent))
                };

                field_to_access.copy_types(type_mapping);
                prefix.copy_types(type_mapping);
            }
            IfLet {
                ref mut variant,
                ref mut enum_type,
                ..
            } => {
                *enum_type = if let Some(matching_id) =
                    look_up_type_id(*enum_type).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*enum_type))
                };
                variant.copy_types(type_mapping);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                *resolved_type_of_parent = if let Some(matching_id) =
                    look_up_type_id(*resolved_type_of_parent).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*resolved_type_of_parent))
                };

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
            TypeProperty { type_id, .. } => {
                *type_id = if let Some(matching_id) =
                    look_up_type_id(*type_id).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*type_id))
                };
            }
            SizeOfValue { expr } => expr.copy_types(type_mapping),
            AbiName(_) => (),
        }
    }
}
