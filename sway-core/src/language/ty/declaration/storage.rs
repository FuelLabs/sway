use std::hash::{Hash, Hasher};

use sway_error::{
    error::{CompileError, StructFieldUsageContext}, handler::{ErrorEmitted, Handler}, warning::{CompileWarning, Warning}
};
use sway_types::{state::StateIndex, Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*, language::{ty::*, Visibility}, transform, type_system::*, Namespace,
};

#[derive(Clone, Debug)]
pub struct TyStorageDecl {
    pub fields: Vec<TyStorageField>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
    pub storage_keyword: Ident,
}

impl Named for TyStorageDecl {
    fn name(&self) -> &Ident {
        &self.storage_keyword
    }
}

impl EqWithEngines for TyStorageDecl {}
impl PartialEqWithEngines for TyStorageDecl {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.fields.eq(&other.fields, engines) && self.attributes == other.attributes
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
    pub fn apply_storage_load(
        &self,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        fields: Vec<Ident>,
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

        let (first_field, remaining_fields) = fields.split_first().expect("Having at least one element in the storage load is guaranteed by the grammar.");

        let (ix, initial_field_type) = match storage_fields
            .iter()
            .enumerate()
            .find(|(_, sf)| &sf.name == first_field)
        {
            Some((ix, TyStorageField { type_argument, .. })) => {
                (StateIndex::new(ix), type_argument.type_id)
            }
            None => {
                return Err(handler.emit_err(CompileError::StorageFieldDoesNotExist {
                    name: first_field.clone(),
                    span: first_field.span(),
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


        let get_struct_decl = |type_id: TypeId| match &*type_engine.get(type_id) {
            TypeInfo::Struct(decl_ref) => {
                Some(decl_engine.get_struct(decl_ref))
            },
            _ => None,
        };

        for field in remaining_fields {
            match get_struct_decl(previous_field_type_id) {
                Some(struct_decl) => {
                    let (struct_can_be_changed, is_public_struct_access) = StructAccessInfo::get_info(&struct_decl, namespace).into();

                    match struct_decl.find_field(field)
                    {
                        Some(struct_field) => {
                            if is_public_struct_access && struct_field.is_private() {
                                // TODO: Uncomment this code and delete the one with warnings once struct field privacy becomes a hard error.
                                // return Err(handler.emit_err(CompileError::StructFieldIsPrivate {
                                //     field_name: field.into(),
                                //     struct_name: struct_decl.call_path.suffix.clone(),
                                //     field_decl_span: struct_field.name.span(),
                                //     struct_can_be_changed,
                                //     usage_context: StructFieldUsageContext::StorageAccess,
                                // }));
                                handler.emit_warn(CompileWarning {
                                    span: field.span(),
                                    warning_content: Warning::StructFieldIsPrivate {
                                        field_name: field.into(),
                                        struct_name: struct_decl.call_path.suffix.clone(),
                                        field_decl_span: struct_field.name.span(),
                                        struct_can_be_changed,
                                        usage_context: StructFieldUsageContext::StorageAccess,
                                    }
                                });
                            }

                            // Everything is fine. Push the storage access descriptor and move to the next field.

                            let current_field_type_id = struct_field.type_argument.type_id;

                            access_descriptors.push(TyStorageAccessDescriptor {
                                name: field.clone(),
                                type_id: current_field_type_id,
                                span: field.span(),
                            });

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
                            let struct_can_be_instantiated = !is_public_struct_access || !struct_decl.has_private_fields();

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
                },
                None => return Err(handler.emit_err(CompileError::FieldAccessOnNonStruct {
                    span: previous_field.span(),
                    actually: engines.help_out(previous_field_type_id).to_string(),
                })),
            };
        }

        let return_type = access_descriptors[access_descriptors.len() - 1].type_id;

        Ok((
            TyStorageAccess {
                fields: access_descriptors,
                ix,
                storage_keyword_span,
            },
            return_type,
        ))
    }

    pub(crate) fn fields_as_typed_struct_fields(&self) -> Vec<TyStructField> {
        self.fields
            .iter()
            .map(
                |TyStorageField {
                     ref name,
                     ref type_argument,
                     ref span,
                     ref attributes,
                     ..
                 }| TyStructField {
                    visibility: Visibility::Public,
                    name: name.clone(),
                    span: span.clone(),
                    type_argument: type_argument.clone(),
                    attributes: attributes.clone(),
                },
            )
            .collect()
    }
}

impl Spanned for TyStorageField {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TyStorageField {
    pub name: Ident,
    pub type_argument: TypeArgument,
    pub initializer: TyExpression,
    pub(crate) span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyStorageField {}
impl PartialEqWithEngines for TyStorageField {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, engines)
            && self.initializer.eq(&other.initializer, engines)
    }
}

impl HashWithEngines for TyStorageField {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStorageField {
            name,
            type_argument,
            initializer,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        type_argument.hash(state, engines);
        initializer.hash(state, engines);
    }
}
