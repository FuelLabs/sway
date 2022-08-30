use crate::{
    declaration_engine::declaration_engine::DeclarationEngine,
    parse_tree::*,
    semantic_analysis::*,
    type_system::*,
    types::{CompileWrapper, ToCompileWrapper},
};

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use std::{collections::HashMap, fmt, fmt::Write};

#[derive(Clone, Debug)]
pub struct ContractCallParams {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TypedExpression>,
}

#[derive(Clone, Debug)]
pub enum TypedExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        call_path: CallPath,
        contract_call_params: HashMap<String, TypedExpression>,
        arguments: Vec<(Ident, TypedExpression)>,
        function_decl: TypedFunctionDeclaration,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        self_state_idx: Option<StateIndex>,
        selector: Option<ContractCallParams>,
    },
    LazyOperator {
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
}

impl PartialEq for CompileWrapper<'_, TypedExpressionVariant> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        match (me, them) {
            (TypedExpressionVariant::Literal(l0), TypedExpressionVariant::Literal(r0)) => l0 == r0,
            (
                TypedExpressionVariant::FunctionApplication {
                    call_path: l_name,
                    arguments: l_arguments,
                    function_decl: l_function_decl,
                    ..
                },
                TypedExpressionVariant::FunctionApplication {
                    call_path: r_name,
                    arguments: r_arguments,
                    function_decl: r_function_decl,
                    ..
                },
            ) => {
                l_name == r_name
                    && l_arguments.len() == r_arguments.len()
                    && l_arguments.iter().map(|(name, _)| name).collect::<Vec<_>>()
                        == r_arguments.iter().map(|(name, _)| name).collect::<Vec<_>>()
                    && l_arguments
                        .iter()
                        .map(|(_, exp)| exp.wrap(de))
                        .collect::<Vec<_>>()
                        == r_arguments
                            .iter()
                            .map(|(_, exp)| exp.wrap(de))
                            .collect::<Vec<_>>()
                    && l_function_decl.body.wrap(de) == r_function_decl.body.wrap(de)
            }
            (
                TypedExpressionVariant::LazyOperator {
                    op: l_op,
                    lhs: l_lhs,
                    rhs: l_rhs,
                },
                TypedExpressionVariant::LazyOperator {
                    op: r_op,
                    lhs: r_lhs,
                    rhs: r_rhs,
                },
            ) => {
                l_op == r_op
                    && (**l_lhs).wrap(de) == (**r_lhs).wrap(de)
                    && (**l_rhs).wrap(de) == (**r_rhs).wrap(de)
            }
            (
                TypedExpressionVariant::VariableExpression {
                    name: l_name,
                    span: l_span,
                    mutability: l_mutability,
                },
                TypedExpressionVariant::VariableExpression {
                    name: r_name,
                    span: r_span,
                    mutability: r_mutability,
                },
            ) => l_name == r_name && l_span == r_span && l_mutability == r_mutability,
            (
                TypedExpressionVariant::Tuple { fields: l_fields },
                TypedExpressionVariant::Tuple { fields: r_fields },
            ) => {
                l_fields.iter().map(|x| x.wrap(de)).collect::<Vec<_>>()
                    == r_fields.iter().map(|x| x.wrap(de)).collect::<Vec<_>>()
            }
            (
                TypedExpressionVariant::Array {
                    contents: l_contents,
                },
                TypedExpressionVariant::Array {
                    contents: r_contents,
                },
            ) => {
                l_contents.iter().map(|x| x.wrap(de)).collect::<Vec<_>>()
                    == r_contents.iter().map(|x| x.wrap(de)).collect::<Vec<_>>()
            }
            (
                TypedExpressionVariant::ArrayIndex {
                    prefix: l_prefix,
                    index: l_index,
                },
                TypedExpressionVariant::ArrayIndex {
                    prefix: r_prefix,
                    index: r_index,
                },
            ) => {
                (**l_prefix).wrap(de) == (**r_prefix).wrap(de)
                    && (**l_index).wrap(de) == (**r_index).wrap(de)
            }
            (
                TypedExpressionVariant::StructExpression {
                    struct_name: l_struct_name,
                    fields: l_fields,
                    span: l_span,
                },
                TypedExpressionVariant::StructExpression {
                    struct_name: r_struct_name,
                    fields: r_fields,
                    span: r_span,
                },
            ) => {
                l_struct_name == r_struct_name
                    && l_fields.wrap(de) == r_fields.wrap(de)
                    && l_span == r_span
            }
            (TypedExpressionVariant::CodeBlock(l), TypedExpressionVariant::CodeBlock(r)) => {
                l.wrap(de) == r.wrap(de)
            }
            (
                TypedExpressionVariant::IfExp {
                    condition: l_condition,
                    then: l_then,
                    r#else: l_r,
                },
                TypedExpressionVariant::IfExp {
                    condition: r_condition,
                    then: r_then,
                    r#else: r_r,
                },
            ) => {
                (**l_condition).wrap(de) == (**r_condition).wrap(de)
                    && (**l_then).wrap(de) == (**r_then).wrap(de)
                    && if let (Some(l), Some(r)) = (l_r, r_r) {
                        (**l).wrap(de) == (**r).wrap(de)
                    } else {
                        true
                    }
            }
            (
                TypedExpressionVariant::AsmExpression {
                    registers: l_registers,
                    body: l_body,
                    returns: l_returns,
                    ..
                },
                TypedExpressionVariant::AsmExpression {
                    registers: r_registers,
                    body: r_body,
                    returns: r_returns,
                    ..
                },
            ) => {
                l_registers.wrap(de) == r_registers.wrap(de)
                    && l_body.clone() == r_body.clone()
                    && l_returns == r_returns
            }
            (
                TypedExpressionVariant::StructFieldAccess {
                    prefix: l_prefix,
                    field_to_access: l_field_to_access,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                TypedExpressionVariant::StructFieldAccess {
                    prefix: r_prefix,
                    field_to_access: r_field_to_access,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix).wrap(de) == (**r_prefix).wrap(de)
                    && l_field_to_access.wrap(de) == r_field_to_access.wrap(de)
                    && look_up_type_id(*l_resolved_type_of_parent).wrap(de)
                        == look_up_type_id(*r_resolved_type_of_parent).wrap(de)
            }
            (
                TypedExpressionVariant::TupleElemAccess {
                    prefix: l_prefix,
                    elem_to_access_num: l_elem_to_access_num,
                    resolved_type_of_parent: l_resolved_type_of_parent,
                    ..
                },
                TypedExpressionVariant::TupleElemAccess {
                    prefix: r_prefix,
                    elem_to_access_num: r_elem_to_access_num,
                    resolved_type_of_parent: r_resolved_type_of_parent,
                    ..
                },
            ) => {
                (**l_prefix).wrap(de) == (**r_prefix).wrap(de)
                    && l_elem_to_access_num == r_elem_to_access_num
                    && look_up_type_id(*l_resolved_type_of_parent).wrap(de)
                        == look_up_type_id(*r_resolved_type_of_parent).wrap(de)
            }
            (
                TypedExpressionVariant::EnumInstantiation {
                    enum_decl: l_enum_decl,
                    variant_name: l_variant_name,
                    tag: l_tag,
                    contents: l_contents,
                    ..
                },
                TypedExpressionVariant::EnumInstantiation {
                    enum_decl: r_enum_decl,
                    variant_name: r_variant_name,
                    tag: r_tag,
                    contents: r_contents,
                    ..
                },
            ) => {
                l_enum_decl.wrap(de) == r_enum_decl.wrap(de)
                    && l_variant_name == r_variant_name
                    && l_tag == r_tag
                    && if let (Some(l_contents), Some(r_contents)) = (l_contents, r_contents) {
                        (**l_contents).wrap(de) == (**r_contents).wrap(de)
                    } else {
                        true
                    }
            }
            (
                TypedExpressionVariant::AbiCast {
                    abi_name: l_abi_name,
                    address: l_address,
                    ..
                },
                TypedExpressionVariant::AbiCast {
                    abi_name: r_abi_name,
                    address: r_address,
                    ..
                },
            ) => l_abi_name == r_abi_name && (**l_address).wrap(de) == (**r_address).wrap(de),
            (
                TypedExpressionVariant::IntrinsicFunction(l_kind),
                TypedExpressionVariant::IntrinsicFunction(r_kind),
            ) => l_kind.wrap(de) == r_kind.wrap(de),
            (
                TypedExpressionVariant::UnsafeDowncast {
                    exp: l_exp,
                    variant: l_variant,
                },
                TypedExpressionVariant::UnsafeDowncast {
                    exp: r_exp,
                    variant: r_variant,
                },
            ) => {
                (**l_exp).wrap(de) == (**r_exp).wrap(de) && l_variant.wrap(de) == r_variant.wrap(de)
            }
            (
                TypedExpressionVariant::EnumTag { exp: l_exp },
                TypedExpressionVariant::EnumTag { exp: r_exp },
            ) => (**l_exp).wrap(de) == (**r_exp).wrap(de),
            (
                TypedExpressionVariant::StorageAccess(l_exp),
                TypedExpressionVariant::StorageAccess(r_exp),
            ) => *l_exp == *r_exp,
            (
                TypedExpressionVariant::WhileLoop {
                    body: l_body,
                    condition: l_condition,
                },
                TypedExpressionVariant::WhileLoop {
                    body: r_body,
                    condition: r_condition,
                },
            ) => {
                l_body.wrap(de) == r_body.wrap(de)
                    && (**l_condition).wrap(de) == (**r_condition).wrap(de)
            }
            _ => false,
        }
    }
}

impl CopyTypes for TypedExpressionVariant {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
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
                    .for_each(|(_ident, expr)| expr.copy_types(type_mapping, de));
                function_decl.copy_types(type_mapping, de);
            }
            LazyOperator { lhs, rhs, .. } => {
                (*lhs).copy_types(type_mapping, de);
                (*rhs).copy_types(type_mapping, de);
            }
            VariableExpression { .. } => (),
            Tuple { fields } => fields
                .iter_mut()
                .for_each(|x| x.copy_types(type_mapping, de)),
            Array { contents } => contents
                .iter_mut()
                .for_each(|x| x.copy_types(type_mapping, de)),
            ArrayIndex { prefix, index } => {
                (*prefix).copy_types(type_mapping, de);
                (*index).copy_types(type_mapping, de);
            }
            StructExpression { fields, .. } => fields
                .iter_mut()
                .for_each(|x| x.copy_types(type_mapping, de)),
            CodeBlock(block) => {
                block.copy_types(type_mapping, de);
            }
            FunctionParameter => (),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.copy_types(type_mapping, de);
                then.copy_types(type_mapping, de);
                if let Some(ref mut r#else) = r#else {
                    r#else.copy_types(type_mapping, de);
                }
            }
            AsmExpression {
                registers, //: Vec<TypedAsmRegisterDeclaration>,
                ..
            } => {
                registers
                    .iter_mut()
                    .for_each(|x| x.copy_types(type_mapping, de));
            }
            // like a variable expression but it has multiple parts,
            // like looking up a field in a struct
            StructFieldAccess {
                prefix,
                field_to_access,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.update_type(type_mapping, de, &field_to_access.span);
                field_to_access.copy_types(type_mapping, de);
                prefix.copy_types(type_mapping, de);
            }
            TupleElemAccess {
                prefix,
                ref mut resolved_type_of_parent,
                ..
            } => {
                resolved_type_of_parent.update_type(type_mapping, de, &prefix.span);
                prefix.copy_types(type_mapping, de);
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                enum_decl.copy_types(type_mapping, de);
                if let Some(ref mut contents) = contents {
                    contents.copy_types(type_mapping, de)
                };
            }
            AbiCast { address, .. } => address.copy_types(type_mapping, de),
            // storage is never generic and cannot be monomorphized
            StorageAccess { .. } => (),
            IntrinsicFunction(kind) => {
                kind.copy_types(type_mapping, de);
            }
            EnumTag { exp } => {
                exp.copy_types(type_mapping, de);
            }
            UnsafeDowncast { exp, variant } => {
                exp.copy_types(type_mapping, de);
                variant.copy_types(type_mapping, de);
            }
            AbiName(_) => (),
            WhileLoop {
                ref mut condition,
                ref mut body,
            } => {
                condition.copy_types(type_mapping, de);
                body.copy_types(type_mapping, de);
            }
            Break => (),
            Continue => (),
            Reassignment(reassignment) => reassignment.copy_types(type_mapping, de),
            StorageReassignment(..) => (),
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
        };
        write!(f, "{}", s)
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

impl PartialEq for CompileWrapper<'_, TypedAsmRegisterDeclaration> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && if let (Some(l), Some(r)) = (me.initializer.clone(), them.initializer.clone()) {
                l.wrap(de) == r.wrap(de)
            } else {
                true
            }
    }
}

impl PartialEq for CompileWrapper<'_, Vec<TypedAsmRegisterDeclaration>> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        if me.len() != them.len() {
            return false;
        }
        me.iter()
            .map(|elem| elem.wrap(de))
            .zip(other.inner.iter().map(|elem| elem.wrap(de)))
            .map(|(left, right)| left == right)
            .all(|elem| elem)
    }
}

impl CopyTypes for TypedAsmRegisterDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping, de: &DeclarationEngine) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping, de)
        }
    }
}
