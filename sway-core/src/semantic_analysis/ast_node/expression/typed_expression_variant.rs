use super::*;
use crate::parse_tree::AsmOp;
use crate::semantic_analysis::ast_node::*;
use crate::type_engine::*;
use crate::Ident;

#[derive(Clone, Debug)]
pub(crate) struct ContractCallMetadata<'sc> {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<TypedExpression<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedExpressionVariant<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: CallPath<'sc>,
        arguments: Vec<(Ident<'sc>, TypedExpression<'sc>)>,
        function_body: TypedCodeBlock<'sc>,
        /// If this is `Some(val)` then `val` is the metadata. If this is `None`, then
        /// there is no selector.
        selector: Option<ContractCallMetadata<'sc>>,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<TypedExpression<'sc>>,
        rhs: Box<TypedExpression<'sc>>,
    },
    VariableExpression {
        name: Ident<'sc>,
    },
    Tuple {
        fields: Vec<TypedExpression<'sc>>,
    },
    Array {
        contents: Vec<TypedExpression<'sc>>,
    },
    ArrayIndex {
        prefix: Box<TypedExpression<'sc>>,
        index: Box<TypedExpression<'sc>>,
    },
    StructExpression {
        struct_name: Ident<'sc>,
        fields: Vec<TypedStructExpressionField<'sc>>,
    },
    CodeBlock(TypedCodeBlock<'sc>),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TypedExpression<'sc>>,
        then: Box<TypedExpression<'sc>>,
        r#else: Option<Box<TypedExpression<'sc>>>,
    },
    AsmExpression {
        registers: Vec<TypedAsmRegisterDeclaration<'sc>>,
        body: Vec<AsmOp<'sc>>,
        returns: Option<(AsmRegister, Span<'sc>)>,
        whole_block_span: Span<'sc>,
    },
    // like a variable expression but it has multiple parts,
    // like looking up a field in a struct
    StructFieldAccess {
        prefix: Box<TypedExpression<'sc>>,
        field_to_access: OwnedTypedStructField,
        field_to_access_span: Span<'sc>,
        resolved_type_of_parent: TypeId,
    },
    EnumArgAccess {
        prefix: Box<TypedExpression<'sc>>,
        //variant_to_access: TypedEnumVariant<'sc>,
        arg_num_to_access: usize,
        resolved_type_of_parent: TypeId,
    },
    EnumInstantiation {
        /// for printing
        enum_decl: TypedEnumDeclaration<'sc>,
        /// for printing
        variant_name: Ident<'sc>,
        tag: usize,
        contents: Option<Box<TypedExpression<'sc>>>,
    },
    AbiCast {
        abi_name: CallPath<'sc>,
        address: Box<TypedExpression<'sc>>,
        #[allow(dead_code)]
        // this span may be used for errors in the future, although it is not right now.
        span: Span<'sc>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAsmRegisterDeclaration<'sc> {
    pub(crate) initializer: Option<TypedExpression<'sc>>,
    pub(crate) name: &'sc str,
    pub(crate) name_span: Span<'sc>,
}

impl TypedAsmRegisterDeclaration<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping)
        }
    }
}

impl<'sc> TypedExpressionVariant<'sc> {
    pub(crate) fn pretty_print(&self) -> String {
        match self {
            TypedExpressionVariant::Literal(lit) => format!(
                "literal {}",
                match lit {
                    Literal::U8(content) => content.to_string(),
                    Literal::U16(content) => content.to_string(),
                    Literal::U32(content) => content.to_string(),
                    Literal::U64(content) => content.to_string(),
                    Literal::String(content) => content.to_string(),
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
            TypedExpressionVariant::EnumArgAccess {
                resolved_type_of_parent,
                arg_num_to_access,
                ..
            } => {
                format!(
                    "\"{}.{}\" arg num access",
                    look_up_type_id(*resolved_type_of_parent).friendly_type_str(),
                    arg_num_to_access
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
                registers, //: Vec<TypedAsmRegisterDeclaration<'sc>>,
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
            EnumArgAccess {
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
        }
    }
}
