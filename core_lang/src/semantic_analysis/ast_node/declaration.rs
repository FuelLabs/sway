use super::impl_trait::Mode;
use super::{
    IsConstant, TypedCodeBlock, TypedExpression, TypedExpressionVariant, TypedReturnStatement,
};
use crate::parse_tree::*;
use crate::semantic_analysis::Namespace;
use crate::span::Span;
use crate::type_engine::*;
use crate::ControlFlowGraph;
use crate::{build_config::BuildConfig, error::*, Ident};

use core_types::{Function, Property};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

mod function;
mod variable;
pub(crate) use function::*;
pub(crate) use variable::*;

#[derive(Clone, Debug)]
pub enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    ConstantDeclaration(TypedConstantDeclaration<'sc>),
    FunctionDeclaration(TypedFunctionDeclaration<'sc>),
    TraitDeclaration(TypedTraitDeclaration<'sc>),
    StructDeclaration(TypedStructDeclaration<'sc>),
    EnumDeclaration(TypedEnumDeclaration<'sc>),
    Reassignment(TypedReassignment<'sc>),
    ImplTrait {
        trait_name: CallPath<'sc>,
        span: Span<'sc>,
        methods: Vec<TypedFunctionDeclaration<'sc>>,
        type_implementing_for: TypeInfo,
    },
    AbiDeclaration(TypedAbiDeclaration<'sc>),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope {
        name: Ident<'sc>,
    },
    ErrorRecovery,
}

impl TypedDeclaration<'_> {
    /// The entry point to monomorphizing typed declarations. Instantiates all new type ids,
    /// assuming `self` has already been copied.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.copy_types(type_mapping),
            ConstantDeclaration(ref mut const_decl) => const_decl.copy_types(type_mapping),
            FunctionDeclaration(ref mut fn_decl) => fn_decl.copy_types(type_mapping),
            TraitDeclaration(ref mut trait_decl) => trait_decl.copy_types(type_mapping),
            StructDeclaration(ref mut struct_decl) => struct_decl.copy_types(type_mapping),
            EnumDeclaration(ref mut enum_decl) => enum_decl.copy_types(type_mapping),
            Reassignment(ref mut reassignment) => reassignment.copy_types(type_mapping),
            ImplTrait {
                ref mut methods, ..
            } => {
                methods.iter_mut().for_each(|x| x.copy_types(type_mapping));
            }
            // generics in an ABI is unsupported by design
            AbiDeclaration(..) => (),
            GenericTypeForFunctionScope { .. } | ErrorRecovery => (),
        }
    }
}

impl<'sc> TypedDeclaration<'sc> {
    /// friendly name string used for error reporting.
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            Reassignment(_) => "reassignment",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery => "error",
        }
    }
    pub(crate) fn return_type(&self) -> CompileResult<'sc, TypeId> {
        ok(
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body, ..
                }) => body.return_type,
                TypedDeclaration::FunctionDeclaration { .. } => {
                    return err(
                        vec![],
                        vec![CompileError::Unimplemented(
                            "Function pointers have not yet been implemented.",
                            self.span(),
                        )],
                    )
                }
                TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                }) => crate::type_engine::insert_type(TypeInfo::Struct {
                    name: name.primary_name.to_string(),
                    fields: fields
                        .iter()
                        .map(TypedStructField::as_owned_typed_struct_field)
                        .collect(),
                }),
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => {
                    rhs.return_type
                }
                TypedDeclaration::GenericTypeForFunctionScope { name } => {
                    insert_type(TypeInfo::UnknownGeneric {
                        name: name.primary_name.to_string(),
                    })
                }
                decl => {
                    return err(
                        vec![],
                        vec![CompileError::NotAType {
                            span: decl.span(),
                            name: decl.pretty_print(),
                            actually_is: decl.friendly_name(),
                        }],
                    )
                }
            },
            vec![],
            vec![],
        )
    }

    pub(crate) fn span(&self) -> Span<'sc> {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span.clone(),
            ConstantDeclaration(TypedConstantDeclaration { name, .. }) => name.span.clone(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span.clone(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span.clone(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            Reassignment(TypedReassignment { lhs, .. }) => {
                lhs.iter().fold(lhs[0].span(), |acc, this| {
                    crate::utils::join_spans(acc, this.span())
                })
            }
            AbiDeclaration(TypedAbiDeclaration { span, .. }) => span.clone(),
            ImplTrait { span, .. } => span.clone(),
            ErrorRecovery | GenericTypeForFunctionScope { .. } => {
                unreachable!("No span exists for these ast node types")
            }
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable,
                    name,
                    ..
                }) => format!(
                    "{} {}",
                    if *is_mutable { "mut" } else { "" },
                    name.primary_name
                ),
                TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                    name, ..
                }) => {
                    name.primary_name.into()
                }
                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::StructDeclaration(TypedStructDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) => lhs
                    .iter()
                    .map(|x| x.name.primary_name)
                    .collect::<Vec<_>>()
                    .join("."),
                _ => String::new(),
            }
        )
    }

    pub(crate) fn visibility(&self) -> Visibility {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(..)
            | GenericTypeForFunctionScope { .. }
            | Reassignment(..)
            | ImplTrait { .. }
            | AbiDeclaration(..)
            | ErrorRecovery => Visibility::Public,
            EnumDeclaration(TypedEnumDeclaration { visibility, .. })
            | ConstantDeclaration(TypedConstantDeclaration { visibility, .. })
            | FunctionDeclaration(TypedFunctionDeclaration { visibility, .. })
            | TraitDeclaration(TypedTraitDeclaration { visibility, .. })
            | StructDeclaration(TypedStructDeclaration { visibility, .. }) => *visibility,
        }
    }
}

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's [AbiDeclaration].
#[derive(Clone, Debug)]
pub struct TypedAbiDeclaration<'sc> {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident<'sc>,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TypedTraitFn<'sc>>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedStructDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) fields: Vec<TypedStructField<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}

impl<'sc> TypedStructDeclaration<'sc> {
    pub(crate) fn monomorphize(&self) -> Self {
        let mut new_decl = self.clone();
        let type_mapping = insert_type_parameters(&self.type_parameters);
        new_decl.copy_types(&type_mapping);
        new_decl
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedStructField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span<'sc>,
}

// TODO(Static span) -- remove this type and use TypedStructField
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OwnedTypedStructField {
    pub(crate) name: String,
    pub(crate) r#type: TypeId,
}

impl OwnedTypedStructField {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }

    pub(crate) fn as_typed_struct_field<'sc>(&self, span: &Span<'sc>) -> TypedStructField<'sc> {
        TypedStructField {
            name: Ident {
                span: span.clone(),
                primary_name: Box::leak(span.clone().as_str().to_string().into_boxed_str()),
            },
            r#type: self.r#type,
            span: span.clone(),
        }
    }

    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.clone(),
            type_field: self.r#type.friendly_type_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}

impl TypedStructField<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
    pub(crate) fn as_owned_typed_struct_field(&self) -> OwnedTypedStructField {
        OwnedTypedStructField {
            name: self.name.primary_name.to_string(),
            r#type: self.r#type,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedEnumDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) variants: Vec<TypedEnumVariant<'sc>>,
    pub(crate) span: Span<'sc>,
    pub(crate) visibility: Visibility,
}
impl TypedEnumDeclaration<'_> {
    pub(crate) fn monomorphize(&self) -> Self {
        let mut new_decl = self.clone();
        let type_mapping = insert_type_parameters(&self.type_parameters);
        new_decl.copy_types(&type_mapping);
        new_decl
    }
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
    /// Returns the [ResolvedType] corresponding to this enum's type.
    pub(crate) fn as_type(&self) -> TypeId {
        crate::type_engine::insert_type(TypeInfo::Enum {
            name: self.name.primary_name.to_string(),
            variant_types: self
                .variants
                .iter()
                .map(TypedEnumVariant::as_owned_typed_enum_variant)
                .collect(),
        })
    }
}
#[derive(Debug, Clone)]
pub struct TypedEnumVariant<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span<'sc>,
}

impl TypedEnumVariant<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
    pub(crate) fn as_owned_typed_enum_variant(&self) -> OwnedTypedEnumVariant {
        OwnedTypedEnumVariant {
            name: self.name.primary_name.to_string(),
            r#type: self.r#type,
            tag: self.tag,
        }
    }
}

// TODO(Static span) -- remove this type and use TypedEnumVariant
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OwnedTypedEnumVariant {
    pub(crate) name: String,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
}

impl OwnedTypedEnumVariant {
    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.clone(),
            type_field: self.r#type.friendly_type_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedConstantDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) value: TypedExpression<'sc>,
    pub(crate) visibility: Visibility,
}

impl TypedConstantDeclaration<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.value.copy_types(type_mapping);
    }
}

// TODO: type check generic type args and their usage
#[derive(Clone, Debug)]
pub struct TypedFunctionDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedCodeBlock<'sc>,
    pub(crate) parameters: Vec<TypedFunctionParameter<'sc>>,
    pub(crate) span: Span<'sc>,
    pub(crate) return_type: TypeId,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub(crate) return_type_span: Span<'sc>,
    pub(crate) visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub(crate) is_contract_call: bool,
}

impl<'sc> TypedFunctionDeclaration<'sc> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.body.copy_types(type_mapping);
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));

        self.return_type = if let Some(matching_id) =
            look_up_type_id(self.return_type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.return_type))
        };
    }
    /// Given a typed function declaration with type parameters, make a copy of it and update the
    /// type ids which refer to generic types to be fresh copies, maintaining their referential
    /// relationship. This is used so when this function is resolved, the types don't clobber the
    /// generic type info.
    pub(crate) fn monomorphize(
        &self,
        type_arguments: Vec<(TypeInfo, Span<'sc>)>,
        self_type: TypeId,
    ) -> CompileResult<'sc, TypedFunctionDeclaration<'sc>> {
        let mut warnings: Vec<CompileWarning> = vec![];
        let mut errors: Vec<CompileError> = vec![];
        debug_assert!(
            !self.type_parameters.is_empty(),
            "Only generic functions can be monomorphized"
        );

        let type_mapping = insert_type_parameters(&self.type_parameters);
        if !type_arguments.is_empty() {
            // check type arguments against parameters
            if self.type_parameters.len() != type_arguments.len() {
                todo!("incorrect number of type args err");
            }

            // check the type arguments
            for ((_, decl_param), (type_argument, type_argument_span)) in
                type_mapping.iter().zip(type_arguments.iter())
            {
                match unify_with_self(
                    *decl_param,
                    insert_type(type_argument.clone()),
                    self_type,
                    type_argument_span,
                ) {
                    Ok(ws) => {
                        for warning in ws {
                            warnings.push(CompileWarning {
                                warning_content: warning,
                                span: type_argument_span.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        errors.push(e.into());
                        continue;
                    }
                }
            }
        }

        let mut new_decl = self.clone();

        // make all type ids fresh ones
        new_decl
            .body
            .contents
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping));

        new_decl
            .parameters
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping));

        new_decl.return_type = if let Some(matching_id) =
            look_up_type_id(new_decl.return_type).matches_type_parameter(&type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(new_decl.return_type))
        };

        ok(new_decl, warnings, errors)
    }
    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span<'sc> {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                self.parameters[0].name.span.clone(),
                |acc, TypedFunctionParameter { type_span, .. }| {
                    crate::utils::join_spans(acc, type_span.clone())
                },
            )
        } else {
            self.name.span.clone()
        }
    }
    pub(crate) fn replace_self_types(self, self_type: TypeId) -> Self {
        TypedFunctionDeclaration {
            name: self.name,
            body: self.body,
            parameters: self
                .parameters
                .iter()
                .map(|x| {
                    let mut x = x.clone();
                    x.r#type = match look_up_type_id(x.r#type) {
                        TypeInfo::SelfType => self_type,
                        _otherwise => x.r#type,
                    };
                    x
                })
                .collect(),
            span: self.span.clone(),
            return_type: match look_up_type_id(self.return_type) {
                TypeInfo::SelfType => self_type,
                _otherwise => self.return_type,
            },
            type_parameters: self.type_parameters.clone(),
            return_type_span: self.return_type_span.clone(),
            visibility: self.visibility,
            is_contract_call: self.is_contract_call,
        }
    }
    pub fn to_fn_selector_value_untruncated(&self) -> CompileResult<'sc, Vec<u8>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut hasher = Sha256::new();
        let data = check!(
            self.to_selector_name(),
            return err(warnings, errors),
            warnings,
            errors
        );
        hasher.update(data);
        let hash = hasher.finalize();
        ok(hash.to_vec(), warnings, errors)
    }
    /// Converts a [TypedFunctionDeclaration] into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four bytes.
    pub fn to_fn_selector_value(&self) -> CompileResult<'sc, [u8; 4]> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let hash = check!(
            self.to_fn_selector_value_untruncated(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // 4 bytes truncation via copying into a 4 byte buffer
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&hash[0..4]);
        ok(buf, warnings, errors)
    }

    pub fn to_selector_name(&self) -> CompileResult<'sc, String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(
                |TypedFunctionParameter {
                     r#type, type_span, ..
                 }| {
                    resolve_type(*r#type, type_span)
                        .expect("unreachable I think?")
                        .to_selector_name(type_span)
                },
            )
            .filter_map(|name| name.ok(&mut warnings, &mut errors))
            .collect::<Vec<String>>();

        ok(
            format!("{}({})", self.name.primary_name, named_params.join(","),),
            warnings,
            errors,
        )
    }

    pub fn generate_json_abi(&self) -> Function {
        Function {
            name: self.name.primary_name.to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.primary_name.to_string(),
                    type_field: x.r#type.friendly_type_str(),
                    components: x.r#type.generate_json_abi(),
                })
                .collect(),
            outputs: vec![Property {
                name: "".to_string(),
                type_field: self.return_type.friendly_type_str(),
                components: self.return_type.generate_json_abi(),
            }],
        }
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::type_engine::IntegerBits;
    let decl = TypedFunctionDeclaration {
        name: Ident {
            primary_name: "foo",
            span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        parameters: vec![],
        span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "foo()".to_string());

    let decl = TypedFunctionDeclaration {
        name: Ident {
            primary_name: "bar",
            span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        body: TypedCodeBlock {
            contents: vec![],
            whole_block_span: Span {
                span: pest::Span::new(" ", 0, 0).unwrap(),
                path: None,
            },
        },
        parameters: vec![
            TypedFunctionParameter {
                name: Ident {
                    primary_name: "foo",
                    span: Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                },
                r#type: crate::type_engine::insert_type(TypeInfo::Str(5)),
                type_span: Span {
                    span: pest::Span::new(" ", 0, 0).unwrap(),
                    path: None,
                },
            },
            TypedFunctionParameter {
                name: Ident {
                    primary_name: "baz",
                    span: Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                },
                r#type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                type_span: Span {
                    span: pest::Span::new(" ", 0, 0).unwrap(),
                    path: None,
                },
            },
        ],
        span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        return_type: 0,
        type_parameters: vec![],
        return_type_span: Span {
            span: pest::Span::new(" ", 0, 0).unwrap(),
            path: None,
        },
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunctionParameter<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
    pub(crate) type_span: Span<'sc>,
}

impl TypedFunctionParameter<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedTraitDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TypedTraitFn<'sc>>,
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}
impl TypedTraitDeclaration<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        let additional_type_map = insert_type_parameters(&self.type_parameters);
        let type_mapping = [type_mapping, &additional_type_map].concat();
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping[..]));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}
#[derive(Clone, Debug)]
pub struct TypedTraitFn<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) parameters: Vec<TypedFunctionParameter<'sc>>,
    pub(crate) return_type: TypeId,
    pub(crate) return_type_span: Span<'sc>,
}

/// Represents the left hand side of a reassignment -- a name to locate it in the
/// namespace, and the type that the name refers to. The type is used for memory layout
/// in asm generation.
#[derive(Clone, Debug)]
pub struct ReassignmentLhs<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: TypeId,
}

impl<'sc> ReassignmentLhs<'sc> {
    pub(crate) fn span(&self) -> Span<'sc> {
        self.name.span.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TypedReassignment<'sc> {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub(crate) lhs: Vec<ReassignmentLhs<'sc>>,
    pub(crate) rhs: TypedExpression<'sc>,
}

impl TypedReassignment<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.rhs.copy_types(type_mapping);
        self.lhs
            .iter_mut()
            .for_each(|ReassignmentLhs { ref mut r#type, .. }| {
                *r#type = if let Some(matching_id) =
                    look_up_type_id(*r#type).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*r#type))
                };
            });
    }
}

#[allow(clippy::too_many_arguments)]
impl<'sc> TypedFunctionDeclaration<'sc> {
    pub fn type_check (
        fn_decl: FunctionDeclaration<'sc>,
        namespace: &mut Namespace<'sc>,
        crate_namespace: Option<&Namespace<'sc>>,
        _return_type_annotation: TypeId,
        _help_text: impl Into<String>,
        // If there are any `Self` types in this declaration,
        // resolve them to this type.
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        mode: Mode,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, TypedFunctionDeclaration<'sc>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let FunctionDeclaration {
            name,
            body,
            parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            ..
        } = fn_decl.clone();
        // insert type parameters as Unknown types
        let type_mapping = insert_type_parameters(&type_parameters);
        let return_type =
            if let Some(matching_id) = return_type.matches_type_parameter(&type_mapping) {
                insert_type(TypeInfo::Ref(matching_id))
            } else {
                namespace
                    .resolve_type_with_self(return_type, self_type)
                    .unwrap_or_else(|_| {
                        errors.push(CompileError::UnknownType {
                            span: return_type_span.clone(),
                        });
                        insert_type(TypeInfo::ErrorRecovery)
                    })
            };

        // insert parameters and generic type declarations into namespace
        let mut namespace = namespace.clone();
        type_parameters.iter().for_each(|param| {
            namespace.insert(param.name_ident.clone(), param.into());
        });
        for FunctionParameter {
            name,
            r#type,
            type_span,
        } in parameters.clone()
        {
            let r#type = if let Some(matching_id) = r#type.matches_type_parameter(&type_mapping) {
                insert_type(TypeInfo::Ref(matching_id))
            } else {
                namespace
                    .resolve_type_with_self(r#type, self_type)
                    .unwrap_or_else(|_| {
                        errors.push(CompileError::UnknownType {
                            span: type_span.clone(),
                        });
                        insert_type(TypeInfo::ErrorRecovery)
                    })
            };
            namespace.insert(
                name.clone(),
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: name.clone(),
                    body: TypedExpression {
                        expression: TypedExpressionVariant::FunctionParameter,
                        return_type: r#type,
                        is_constant: IsConstant::No,
                        span: name.span.clone(),
                    },
                    is_mutable: false, // TODO allow mutable function params?
                    type_ascription: r#type,
                }),
            );
        }

        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = check!(
            TypedCodeBlock::type_check(
                body.clone(),
                &namespace,
                crate_namespace,
                return_type,
                "Function body's return type does not match up with its return type annotation.",
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
            ),
            (
                TypedCodeBlock {
                    contents: vec![],
                    whole_block_span: body.whole_block_span.clone()
                },
                crate::type_engine::insert_type(TypeInfo::ErrorRecovery)
            ),
            warnings,
            errors
        );

        let parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     r#type,
                     type_span,
                 }| TypedFunctionParameter {
                    name,
                    r#type: if let Some(matching_id) = r#type.matches_type_parameter(&type_mapping)
                    {
                        insert_type(TypeInfo::Ref(matching_id))
                    } else {
                        namespace
                            .resolve_type_with_self(r#type, self_type)
                            .unwrap_or_else(|_| {
                                errors.push(CompileError::UnknownType {
                                    span: type_span.clone(),
                                });
                                insert_type(TypeInfo::ErrorRecovery)
                            })
                    },
                    type_span,
                },
            )
            .collect::<Vec<_>>();
        // handle the return statement(s)
        let return_statements: Vec<(&TypedExpression, &Span<'sc>)> = body
            .contents
            .iter()
            .filter_map(|x| {
                if let crate::semantic_analysis::TypedAstNode {
                    content:
                        crate::semantic_analysis::TypedAstNodeContent::ReturnStatement(
                            TypedReturnStatement { ref expr },
                        ),
                    span,
                } = x
                {
                    Some((expr, span))
                } else {
                    None
                }
            })
            .collect();
        for (stmt, span) in return_statements {
            match crate::type_engine::unify_with_self(
                stmt.return_type,
                return_type,
                self_type,
                span,
            ) {
                Ok(ws) => {
                    for warning in ws {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: span.clone(),
                        });
                    }
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                } //    "Function body's return type does not match up with its return type annotation.",
            }
        }

        // if this is an abi function, it is required that it begins with
        // the three parameters related to contract calls
        //  gas_to_forward: u64,
        //  coins_to_forward: u64,
        //  color_of_coins: b256,
        //
        //  eventually this will be a `ContractRequest`
        //
        //  not spending _too_ much time on particularly specific error messages here since
        //  it is a temporary workaround
        if mode == Mode::ImplAbiFn {
            if parameters.len() == 4 {
                if look_up_type_id(parameters[0].r#type)
                    != TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[0].type_span.clone(),
                    });
                }
                if look_up_type_id(parameters[1].r#type)
                    != TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[1].type_span.clone(),
                    });
                }
                if look_up_type_id(parameters[2].r#type) != TypeInfo::B256 {
                    errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                        span: parameters[2].type_span.clone(),
                    });
                }
            } else {
                errors.push(CompileError::AbiFunctionRequiresSpecificSignature {
                    span: parameters
                        .get(0)
                        .map(|x| x.type_span.clone())
                        .unwrap_or_else(|| fn_decl.name.span.clone()),
                });
            }
        }

        ok(
            TypedFunctionDeclaration {
                name,
                body,
                parameters,
                span: span.clone(),
                return_type,
                type_parameters,
                return_type_span,
                visibility,
                // if this is for a contract, then it is a contract call
                is_contract_call: mode == Mode::ImplAbiFn,
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> TypedTraitFn<'sc> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.return_type = if let Some(matching_id) =
            look_up_type_id(self.return_type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.return_type))
        };
    }
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TypedFunctionDeclaration<'sc> {
        TypedFunctionDeclaration {
            name: self.name.clone(),
            body: TypedCodeBlock {
                contents: vec![],
                whole_block_span: self.name.span.clone(),
            },
            parameters: self.parameters.clone(),
            span: self.name.span.clone(),
            return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}
