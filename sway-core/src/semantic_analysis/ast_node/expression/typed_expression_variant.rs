use crate::{parse_tree::*, semantic_analysis::*, type_system::*};

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use derivative::Derivative;
use std::{collections::HashMap, fmt, fmt::Write};

#[derive(Clone, Debug)]
pub struct ContractCallParams {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TypedExpression>,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Eq)]
pub enum TypedExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        call_path: CallPath,
        #[derivative(Eq(bound = ""))]
        contract_call_params: HashMap<String, TypedExpression>,
        arguments: Vec<(Ident, TypedExpression)>,
        function_decl: TypedFunctionDeclaration,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        self_state_idx: Option<StateIndex>,
        #[derivative(Eq(bound = ""))]
        selector: Option<ContractCallParams>,
    },
    LazyOperator {
        #[derivative(Eq(bound = ""))]
        op: LazyOp,
        lhs: Box<TypedExpression>,
        rhs: Box<TypedExpression>,
    },
    VariableExpression {
        name: Ident,
        span: Span,
        mutability: VariableMutability,
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
        span: Span,
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
        field_instantiation_span: Span,
        resolved_type_of_parent: TypeId,
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
        /// use these spans as it points to the call site and not the declaration.
        /// They are also used in the language server.
        enum_instantiation_span: Span,
        variant_instantiation_span: Span,
    },
    AbiCast {
        abi_name: CallPath,
        address: Box<TypedExpression>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span,
    },
    StorageAccess(TypeCheckedStorageAccess),
    IntrinsicFunction(TypedIntrinsicFunctionKind),
    /// a zero-sized type-system-only compile-time thing that is used for constructing ABI casts.
    AbiName(AbiName),
    /// grabs the enum tag from the particular enum and variant of the `exp`
    EnumTag {
        exp: Box<TypedExpression>,
    },
    /// performs an unsafe cast from the `exp` to the type of the given enum `variant`
    UnsafeDowncast {
        exp: Box<TypedExpression>,
        variant: TypedEnumVariant,
    },
    WhileLoop {
        condition: Box<TypedExpression>,
        body: TypedCodeBlock,
    },
    Break,
    Continue,
    Reassignment(Box<TypedReassignment>),
    StorageReassignment(Box<TypeCheckedStorageReassignment>),
    Return(Box<TypedReturnStatement>),
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
                    call_path: l_name,
                    arguments: l_arguments,
                    function_decl: l_function_decl,
                    ..
                },
                Self::FunctionApplication {
                    call_path: r_name,
                    arguments: r_arguments,
                    function_decl: r_function_decl,
                    ..
                },
            ) => {
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

impl CopyTypes for TypedExpressionVariant {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        use TypedExpressionVariant::*;
        match self {
            Literal(..) => (),
            FunctionApplication {
                arguments,
                function_decl,
                ..
            } => {
                arguments
                    .iter_mut()
                    .for_each(|(_ident, expr)| expr.copy_types(type_mapping));
                function_decl.copy_types(type_mapping);
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

impl fmt::Display for TypedExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TypedExpressionVariant::Literal(lit) => format!("literal {}", lit),
            TypedExpressionVariant::FunctionApplication {
                call_path: name, ..
            } => {
                format!("\"{}\" fn entry", name.suffix.as_str())
            }
            TypedExpressionVariant::LazyOperator { op, .. } => match op {
                LazyOp::And => "&&".into(),
                LazyOp::Or => "||".into(),
            },
            TypedExpressionVariant::Tuple { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| field.to_string())
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
                    look_up_type_id(*resolved_type_of_parent),
                    field_to_access.name
                )
            }
            TypedExpressionVariant::TupleElemAccess {
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
            TypedExpressionVariant::IntrinsicFunction(kind) => kind.to_string(),
            TypedExpressionVariant::AbiName(n) => format!("ABI name {}", n),
            TypedExpressionVariant::EnumTag { exp } => {
                format!("({} as tag)", look_up_type_id(exp.return_type))
            }
            TypedExpressionVariant::UnsafeDowncast { exp, variant } => {
                format!("({} as {})", look_up_type_id(exp.return_type), variant.name)
            }
            TypedExpressionVariant::WhileLoop { condition, .. } => {
                format!("while loop on {}", condition)
            }
            TypedExpressionVariant::Break => "break".to_string(),
            TypedExpressionVariant::Continue => "continue".to_string(),
            TypedExpressionVariant::Reassignment(reassignment) => {
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
            TypedExpressionVariant::StorageReassignment(storage_reassignment) => {
                let place: String = {
                    storage_reassignment
                        .fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect()
                };
                format!("storage reassignment to {}", place)
            }
            TypedExpressionVariant::Return(stmt) => {
                format!("return {}", stmt.expr)
            }
        };
        write!(f, "{}", s)
    }
}

impl TypedExpressionVariant {
    /// Returns `self` as a literal, if possible.
    pub(crate) fn extract_literal_value(&self) -> Option<Literal> {
        match self {
            TypedExpressionVariant::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }
}

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedStorageAccess {
    pub fields: Vec<TypeCheckedStorageAccessDescriptor>,
    pub(crate) ix: StateIndex,
}

impl Spanned for TypeCheckedStorageAccess {
    fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
}

impl TypeCheckedStorageAccess {
    pub fn storage_field_name(&self) -> Ident {
        self.fields[0].name.clone()
    }
}

/// Describes a single subfield access in the sequence when accessing a subfield within storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedStorageAccessDescriptor {
    pub name: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub struct TypedAsmRegisterDeclaration {
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

impl CopyTypes for TypedAsmRegisterDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping)
        }
    }
}
