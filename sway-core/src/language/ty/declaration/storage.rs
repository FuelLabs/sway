use crate::{
    engine_threading::*,
    ir_generation::storage::get_storage_key_string,
    language::parsed::StorageDeclaration,
    transform::{self},
    ty::*,
    type_system::*,
    Namespace,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_error::{
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyStorageDecl {
    pub fields: Vec<TyStorageField>,
    pub span: Span,
    pub attributes: transform::Attributes,
    pub storage_keyword: Ident,
}

impl TyDeclParsedType for TyStorageDecl {
    type ParsedType = StorageDeclaration;
}

impl Named for TyStorageDecl {
    fn name(&self) -> &Ident {
        &self.storage_keyword
    }
}

impl EqWithEngines for TyStorageDecl {}
impl PartialEqWithEngines for TyStorageDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.fields.eq(&other.fields, ctx) && self.attributes == other.attributes
    }
}

impl HashWithEngines for TyStorageDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStorageDecl {
            fields,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
            storage_keyword: _,
        } = self;
        fields.hash(state, engines);
    }
}

impl Spanned for TyStorageDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl TyStorageDecl {
    /// Given a path that consists of `fields`, where the first field is one of the storage fields,
    /// find the type information of all the elements in the path and return it as a [TyStorageAccess].
    ///
    /// The first element in the `fields` must be one of the storage fields.
    /// The last element in the `fields` can, but must not be, a struct.
    /// All the elements in between must be structs.
    ///
    /// An error is returned if the above constraints are violated or if the access to the struct fields
    /// fails. E.g, if the struct field does not exists or is an inaccessible private field.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_storage_load(
        &self,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        namespace_names: &[Ident],
        fields: &[Ident],
        storage_fields: &[TyStorageField],
        storage_keyword_span: Span,
    ) -> Result<(TyStorageAccess, TypeId), ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();

        // The resulting storage access descriptors, built on the go as we move through the `fields`.
        let mut access_descriptors = vec![];
        // The field we've analyzed before the current field we are on, and its type id.
        let mut previous_field: &Ident;
        let mut previous_field_type_id: TypeId;

        let (first_field, remaining_fields) = fields.split_first().expect(
            "Having at least one element in the storage load is guaranteed by the grammar.",
        );

        let (initial_field_type, initial_field_key, initial_field_name) =
            match storage_fields.iter().find(|sf| {
                &sf.name == first_field
                    && sf.namespace_names.len() == namespace_names.len()
                    && sf
                        .namespace_names
                        .iter()
                        .zip(namespace_names.iter())
                        .all(|(n1, n2)| n1 == n2)
            }) {
                Some(TyStorageField {
                    type_argument,
                    key_expression,
                    name,
                    ..
                }) => (type_argument.type_id(), key_expression, name),
                None => {
                    return Err(handler.emit_err(CompileError::StorageFieldDoesNotExist {
                        field_name: first_field.into(),
                        available_fields: storage_fields
                            .iter()
                            .map(|sf| (sf.namespace_names.clone(), sf.name.clone()))
                            .collect(),
                        storage_decl_span: self.span(),
                    }));
                }
            };

        access_descriptors.push(TyStorageAccessDescriptor {
            name: first_field.clone(),
            type_id: initial_field_type,
            span: first_field.span(),
        });

        previous_field = first_field;
        previous_field_type_id = initial_field_type;

        // Storage cannot contain references, so there is no need for checking
        // if the declaration is a reference to a struct. References can still
        // be erroneously declared in the storage, and the type behind a concrete
        // field access might be a reference to struct, but we do not treat that
        // as a special case but just another one "not a struct".
        // The FieldAccessOnNonStruct error message will explain that in the case
        // of storage access, fields can be accessed only on structs.
        let get_struct_decl = |type_id: TypeId| match &*type_engine.get(type_id) {
            TypeInfo::Struct(decl_ref) => Some(decl_engine.get_struct(decl_ref)),
            _ => None,
        };

        let mut struct_field_names = vec![];

        for field in remaining_fields {
            match get_struct_decl(previous_field_type_id) {
                Some(struct_decl) => {
                    let (struct_can_be_changed, is_public_struct_access) =
                        StructAccessInfo::get_info(engines, &struct_decl, namespace).into();

                    match struct_decl.find_field(field) {
                        Some(struct_field) => {
                            if is_public_struct_access && struct_field.is_private() {
                                return Err(handler.emit_err(CompileError::StructFieldIsPrivate {
                                    field_name: field.into(),
                                    struct_name: struct_decl.call_path.suffix.clone(),
                                    field_decl_span: struct_field.name.span(),
                                    struct_can_be_changed,
                                    usage_context: StructFieldUsageContext::StorageAccess,
                                }));
                            }

                            // Everything is fine. Push the storage access descriptor and move to the next field.

                            let current_field_type_id = struct_field.type_argument.type_id;

                            access_descriptors.push(TyStorageAccessDescriptor {
                                name: field.clone(),
                                type_id: current_field_type_id,
                                span: field.span(),
                            });

                            struct_field_names.push(field.as_str().to_string());

                            previous_field = field;
                            previous_field_type_id = current_field_type_id;
                        }
                        None => {
                            // Since storage cannot be passed to other modules, the access
                            // is always in the module of the storage declaration.
                            // If the struct cannot be instantiated in this module at all,
                            // we will just show the error, without any additional help lines
                            // showing available fields or anything.
                            // Note that if the struct is empty it can always be instantiated.
                            let struct_can_be_instantiated =
                                !is_public_struct_access || !struct_decl.has_private_fields();

                            let available_fields = if struct_can_be_instantiated {
                                struct_decl.accessible_fields_names(is_public_struct_access)
                            } else {
                                vec![]
                            };

                            return Err(handler.emit_err(CompileError::StructFieldDoesNotExist {
                                field_name: field.into(),
                                available_fields,
                                is_public_struct_access,
                                struct_name: struct_decl.call_path.suffix.clone(),
                                struct_decl_span: struct_decl.span(),
                                struct_is_empty: struct_decl.is_empty(),
                                usage_context: StructFieldUsageContext::StorageAccess,
                            }));
                        }
                    }
                }
                None => {
                    return Err(handler.emit_err(CompileError::FieldAccessOnNonStruct {
                        actually: engines.help_out(previous_field_type_id).to_string(),
                        storage_variable: Some(previous_field.to_string()),
                        field_name: field.into(),
                        span: previous_field.span(),
                    }))
                }
            };
        }

        let return_type = access_descriptors[access_descriptors.len() - 1].type_id;

        Ok((
            TyStorageAccess {
                fields: access_descriptors,
                key_expression: initial_field_key.clone().map(Box::new),
                storage_field_names: namespace_names
                    .iter()
                    .map(|n| n.as_str().to_string())
                    .chain(vec![initial_field_name.as_str().to_string()])
                    .collect(),
                struct_field_names,
                storage_keyword_span,
            },
            return_type,
        ))
    }
}

impl Spanned for TyStorageField {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyStorageField {
    pub name: Ident,
    pub namespace_names: Vec<Ident>,
    pub key_expression: Option<TyExpression>,
    pub type_argument: GenericArgument,
    pub initializer: TyExpression,
    pub(crate) span: Span,
    pub attributes: transform::Attributes,
}

impl TyStorageField {
    /// Returns the full name of the [TyStorageField], consisting
    /// of its name preceded by its full namespace path.
    /// E.g., "storage::ns1::ns1.name".
    pub fn full_name(&self) -> String {
        get_storage_key_string(
            &self
                .namespace_names
                .iter()
                .map(|i| i.as_str().to_string())
                .chain(vec![self.name.as_str().to_string()])
                .collect::<Vec<_>>(),
        )
    }
}

impl EqWithEngines for TyStorageField {}
impl PartialEqWithEngines for TyStorageField {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.namespace_names.eq(&other.namespace_names)
            && self.type_argument.eq(&other.type_argument, ctx)
            && self.initializer.eq(&other.initializer, ctx)
    }
}

impl HashWithEngines for TyStorageField {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStorageField {
            name,
            namespace_names,
            key_expression,
            type_argument,
            initializer,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        namespace_names.hash(state);
        key_expression.hash(state, engines);
        type_argument.hash(state, engines);
        initializer.hash(state, engines);
    }
}
