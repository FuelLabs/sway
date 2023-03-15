use std::{
    collections::HashMap,
    fmt::{self, Write},
    hash::{Hash, Hasher},
};

use sway_types::{state::StateIndex, Ident, Span};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{ty::*, *},
    type_system::*,
};

#[derive(Clone, Debug)]
pub enum TyExpressionVariant {
    Literal(Literal),
    FunctionApplication {
        call_path: CallPath,
        contract_call_params: HashMap<String, TyExpression>,
        arguments: Vec<(Ident, TyExpression)>,
        fn_ref: DeclRefFunction,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        self_state_idx: Option<StateIndex>,
        selector: Option<ContractCallParams>,
        /// optional binding information for the LSP
        type_binding: Option<TypeBinding<()>>,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<TyExpression>,
        rhs: Box<TyExpression>,
    },
    VariableExpression {
        name: Ident,
        span: Span,
        mutability: VariableMutability,
        call_path: Option<CallPath>,
    },
    Tuple {
        fields: Vec<TyExpression>,
    },
    Array {
        elem_type: TypeId,
        contents: Vec<TyExpression>,
    },
    ArrayIndex {
        prefix: Box<TyExpression>,
        index: Box<TyExpression>,
    },
    StructExpression {
        struct_ref: DeclRef<DeclId<TyStructDecl>>,
        fields: Vec<TyStructExpressionField>,
        instantiation_span: Span,
        call_path_binding: TypeBinding<CallPath>,
    },
    CodeBlock(TyCodeBlock),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    MatchExp {
        desugared: Box<TyExpression>,
        scrutinees: Vec<TyScrutinee>,
    },
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
        enum_ref: DeclRef<DeclId<TyEnumDecl>>,
        /// for printing
        variant_name: Ident,
        tag: usize,
        contents: Option<Box<TyExpression>>,
        /// If there is an error regarding this instantiation of the enum,
        /// use these spans as it points to the call site and not the declaration.
        /// They are also used in the language server.
        variant_instantiation_span: Span,
        call_path_binding: TypeBinding<CallPath>,
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

impl EqWithEngines for TyExpressionVariant {}
impl PartialEqWithEngines for TyExpressionVariant {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (
                Self::FunctionApplication {
                    call_path: l_name,
                    arguments: l_arguments,
                    fn_ref: l_fn_ref,
                    ..
                },
                Self::FunctionApplication {
                    call_path: r_name,
                    arguments: r_arguments,
                    fn_ref: r_fn_ref,
                    ..
                },
            ) => {
                l_name == r_name
                    && l_arguments.len() == r_arguments.len()
                    && l_arguments
                        .iter()
                        .zip(r_arguments.iter())
                        .all(|((xa, xb), (ya, yb))| xa == ya && xb.eq(yb, engines))
                    && l_fn_ref.eq(r_fn_ref, engines)
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
            ) => {
                l_op == r_op
                    && (**l_lhs).eq(&(**r_lhs), engines)
                    && (**l_rhs).eq(&(**r_rhs), engines)
            }
            (
                Self::VariableExpression {
                    name: l_name,
                    span: l_span,
                    mutability: l_mutability,
                    call_path: _,
                },
                Self::VariableExpression {
                    name: r_name,
                    span: r_span,
                    mutability: r_mutability,
                    call_path: _,
                },
            ) => l_name == r_name && l_span == r_span && l_mutability == r_mutability,
            (Self::Tuple { fields: l_fields }, Self::Tuple { fields: r_fields }) => {
                l_fields.eq(r_fields, engines)
            }
            (
                Self::Array {
                    contents: l_contents,
                    ..
                },
                Self::Array {
                    contents: r_contents,
                    ..
                },
            ) => l_contents.eq(r_contents, engines),
            (
                Self::ArrayIndex {
                    prefix: l_prefix,
                    index: l_index,
                },
                Self::ArrayIndex {
                    prefix: r_prefix,
                    index: r_index,
                },
            ) => (**l_prefix).eq(&**r_prefix, engines) && (**l_index).eq(&**r_index, engines),
            (
                Self::StructExpression {
                    struct_ref: l_struct_ref,
                    fields: l_fields,
                    instantiation_span: l_span,
                    call_path_binding: _,
                },
                Self::StructExpression {
                    struct_ref: r_struct_ref,
                    fields: r_fields,
                    instantiation_span: r_span,
                    call_path_binding: _,
                },
            ) => {
                l_struct_ref.eq(r_struct_ref, engines)
                    && l_fields.eq(r_fields, engines)
                    && l_span == r_span
            }
            (Self::CodeBlock(l0), Self::CodeBlock(r0)) => l0.eq(r0, engines),
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
                (**l_condition).eq(&**r_condition, engines)
                    && (**l_then).eq(&**r_then, engines)
                    && if let (Some(l), Some(r)) = (l_r, r_r) {
                        (**l).eq(&**r, engines)
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
                l_registers.eq(r_registers, engines)
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
                (**l_prefix).eq(&**r_prefix, engines)
                    && l_field_to_access.eq(r_field_to_access, engines)
                    && type_engine
                        .get(*l_resolved_type_of_parent)
                        .eq(&type_engine.get(*r_resolved_type_of_parent), engines)
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
                (**l_prefix).eq(&**r_prefix, engines)
                    && l_elem_to_access_num == r_elem_to_access_num
                    && type_engine
                        .get(*l_resolved_type_of_parent)
                        .eq(&type_engine.get(*r_resolved_type_of_parent), engines)
            }
            (
                Self::EnumInstantiation {
                    enum_ref: l_enum_ref,
                    variant_name: l_variant_name,
                    tag: l_tag,
                    contents: l_contents,
                    ..
                },
                Self::EnumInstantiation {
                    enum_ref: r_enum_ref,
                    variant_name: r_variant_name,
                    tag: r_tag,
                    contents: r_contents,
                    ..
                },
            ) => {
                l_enum_ref.eq(r_enum_ref, engines)
                    && l_variant_name == r_variant_name
                    && l_tag == r_tag
                    && if let (Some(l_contents), Some(r_contents)) = (l_contents, r_contents) {
                        (**l_contents).eq(&**r_contents, engines)
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
            ) => l_abi_name == r_abi_name && (**l_address).eq(&**r_address, engines),
            (Self::IntrinsicFunction(l_kind), Self::IntrinsicFunction(r_kind)) => {
                l_kind.eq(r_kind, engines)
            }
            (
                Self::UnsafeDowncast {
                    exp: l_exp,
                    variant: l_variant,
                },
                Self::UnsafeDowncast {
                    exp: r_exp,
                    variant: r_variant,
                },
            ) => l_exp.eq(r_exp, engines) && l_variant.eq(r_variant, engines),
            (Self::EnumTag { exp: l_exp }, Self::EnumTag { exp: r_exp }) => {
                l_exp.eq(&**r_exp, engines)
            }
            (Self::StorageAccess(l_exp), Self::StorageAccess(r_exp)) => l_exp.eq(r_exp, engines),
            (
                Self::WhileLoop {
                    body: l_body,
                    condition: l_condition,
                },
                Self::WhileLoop {
                    body: r_body,
                    condition: r_condition,
                },
            ) => l_body.eq(r_body, engines) && l_condition.eq(r_condition, engines),
            (l, r) => std::mem::discriminant(l) == std::mem::discriminant(r),
        }
    }
}

impl HashWithEngines for TyExpressionVariant {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let type_engine = engines.te();
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Literal(lit) => {
                lit.hash(state);
            }
            Self::FunctionApplication {
                call_path,
                arguments,
                fn_ref,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                contract_call_params: _,
                self_state_idx: _,
                selector: _,
                type_binding: _,
            } => {
                call_path.hash(state);
                fn_ref.hash(state, engines);
                arguments.iter().for_each(|(name, arg)| {
                    name.hash(state);
                    arg.hash(state, engines);
                });
            }
            Self::LazyOperator { op, lhs, rhs } => {
                op.hash(state);
                lhs.hash(state, engines);
                rhs.hash(state, engines);
            }
            Self::VariableExpression {
                name,
                mutability,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                call_path: _,
                span: _,
            } => {
                name.hash(state);
                mutability.hash(state);
            }
            Self::Tuple { fields } => {
                fields.hash(state, engines);
            }
            Self::Array {
                contents,
                elem_type: _,
            } => {
                contents.hash(state, engines);
            }
            Self::ArrayIndex { prefix, index } => {
                prefix.hash(state, engines);
                index.hash(state, engines);
            }
            Self::StructExpression {
                struct_ref,
                fields,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                instantiation_span: _,
                call_path_binding: _,
            } => {
                struct_ref.hash(state, engines);
                fields.hash(state, engines);
            }
            Self::CodeBlock(contents) => {
                contents.hash(state, engines);
            }
            Self::MatchExp {
                desugared,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                scrutinees: _,
            } => {
                desugared.hash(state, engines);
            }
            Self::IfExp {
                condition,
                then,
                r#else,
            } => {
                condition.hash(state, engines);
                then.hash(state, engines);
                if let Some(x) = r#else.as_ref() {
                    x.hash(state, engines)
                }
            }
            Self::AsmExpression {
                registers,
                body,
                returns,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                whole_block_span: _,
            } => {
                registers.hash(state, engines);
                body.hash(state);
                returns.hash(state);
            }
            Self::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                field_instantiation_span: _,
            } => {
                prefix.hash(state, engines);
                field_to_access.hash(state, engines);
                type_engine
                    .get(*resolved_type_of_parent)
                    .hash(state, engines);
            }
            Self::TupleElemAccess {
                prefix,
                elem_to_access_num,
                resolved_type_of_parent,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                elem_to_access_span: _,
            } => {
                prefix.hash(state, engines);
                elem_to_access_num.hash(state);
                type_engine
                    .get(*resolved_type_of_parent)
                    .hash(state, engines);
            }
            Self::EnumInstantiation {
                enum_ref,
                variant_name,
                tag,
                contents,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                variant_instantiation_span: _,
                call_path_binding: _,
            } => {
                enum_ref.hash(state, engines);
                variant_name.hash(state);
                tag.hash(state);
                if let Some(x) = contents.as_ref() {
                    x.hash(state, engines)
                }
            }
            Self::AbiCast {
                abi_name,
                address,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                span: _,
            } => {
                abi_name.hash(state);
                address.hash(state, engines);
            }
            Self::StorageAccess(exp) => {
                exp.hash(state, engines);
            }
            Self::IntrinsicFunction(exp) => {
                exp.hash(state, engines);
            }
            Self::AbiName(name) => {
                name.hash(state);
            }
            Self::EnumTag { exp } => {
                exp.hash(state, engines);
            }
            Self::UnsafeDowncast { exp, variant } => {
                exp.hash(state, engines);
                variant.hash(state, engines);
            }
            Self::WhileLoop { condition, body } => {
                condition.hash(state, engines);
                body.hash(state, engines);
            }
            Self::Break | Self::Continue | Self::FunctionParameter => {}
            Self::Reassignment(exp) => {
                exp.hash(state, engines);
            }
            Self::StorageReassignment(exp) => {
                exp.hash(state, engines);
            }
            Self::Return(exp) => {
                exp.hash(state, engines);
            }
        }
    }
}

impl DisplayWithEngines for TyExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        // TODO: Implement user-friendly display strings if needed.
        DebugWithEngines::fmt(self, f, engines)
    }
}

impl DebugWithEngines for TyExpressionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        let s = match self {
            TyExpressionVariant::Literal(lit) => format!("literal {lit}"),
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
                    .map(|field| format!("{:?}", engines.help_out(field)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple({fields})")
            }
            TyExpressionVariant::Array { .. } => "array".into(),
            TyExpressionVariant::ArrayIndex { .. } => "[..]".into(),
            TyExpressionVariant::StructExpression { struct_ref, .. } => {
                format!("\"{}\" struct init", struct_ref.name().as_str())
            }
            TyExpressionVariant::CodeBlock(_) => "code block entry".into(),
            TyExpressionVariant::FunctionParameter => "fn param access".into(),
            TyExpressionVariant::MatchExp { .. } | TyExpressionVariant::IfExp { .. } => {
                "if exp".into()
            }
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
                    "\"{:?}.{}\" struct field access",
                    engines.help_out(*resolved_type_of_parent),
                    field_to_access.name
                )
            }
            TyExpressionVariant::TupleElemAccess {
                resolved_type_of_parent,
                elem_to_access_num,
                ..
            } => {
                format!(
                    "\"{:?}.{}\" tuple index",
                    engines.help_out(*resolved_type_of_parent),
                    elem_to_access_num
                )
            }
            TyExpressionVariant::VariableExpression { name, .. } => {
                format!("\"{}\" variable exp", name.as_str())
            }
            TyExpressionVariant::EnumInstantiation {
                tag,
                enum_ref,
                variant_name,
                ..
            } => {
                format!(
                    "{}::{} enum instantiation (tag: {})",
                    enum_ref.name().as_str(),
                    variant_name.as_str(),
                    tag
                )
            }
            TyExpressionVariant::StorageAccess(access) => {
                format!("storage field {} access", access.storage_field_name())
            }
            TyExpressionVariant::IntrinsicFunction(kind) => format!("{:?}", engines.help_out(kind)),
            TyExpressionVariant::AbiName(n) => format!("ABI name {n}"),
            TyExpressionVariant::EnumTag { exp } => {
                format!("({:?} as tag)", engines.help_out(exp.return_type))
            }
            TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                format!(
                    "({:?} as {})",
                    engines.help_out(exp.return_type),
                    variant.name
                )
            }
            TyExpressionVariant::WhileLoop { condition, .. } => {
                format!("while loop on {:?}", engines.help_out(&**condition))
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
                            write!(&mut place, "{index}").unwrap();
                        }
                        ProjectionKind::ArrayIndex { index, .. } => {
                            write!(&mut place, "{index:#?}").unwrap();
                        }
                    }
                }
                format!("reassignment to {place}")
            }
            TyExpressionVariant::StorageReassignment(storage_reassignment) => {
                let place: String = {
                    storage_reassignment
                        .fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect()
                };
                format!("storage reassignment to {place}")
            }
            TyExpressionVariant::Return(exp) => {
                format!("return {:?}", engines.help_out(&**exp))
            }
        };
        write!(f, "{s}")
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
            TyExpressionVariant::MatchExp { desugared, .. } => {
                desugared.expression.gather_return_statements()
            }
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
            TyExpressionVariant::Array {
                elem_type: _,
                contents,
            } => contents
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
