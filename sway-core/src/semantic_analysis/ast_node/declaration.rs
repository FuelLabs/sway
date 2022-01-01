use super::impl_trait::Mode;
use super::{TypedCodeBlock, TypedExpression};
use crate::parse_tree::*;
use crate::span::Span;
use crate::type_engine::*;
use crate::{error::*, Ident};
use sway_types::Property;

mod function;
mod variable;
pub use function::*;
pub use variable::*;

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
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => rhs.return_type,
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

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's `AbiDeclaration`.
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
            type_field: self.r#type.json_abi_str(),
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
            type_field: self.r#type.json_abi_str(),
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
            purity: Default::default(),
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
