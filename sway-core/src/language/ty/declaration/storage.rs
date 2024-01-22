use std::hash::{Hash, Hasher};

use sway_error::{
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{state::StateIndex, Ident, Named, Span, Spanned};

use crate::{
    decl_engine::DeclEngine, engine_threading::*, language::{ty::*, Visibility}, transform, type_system::*, Namespace,
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
    /// Given a field, find its type information in the declaration and return it. If the field has not
    /// been declared as a part of storage, return an error.
    pub fn apply_storage_load(
        &self,
        handler: &Handler,
        type_engine: &TypeEngine,
        decl_engine: &DeclEngine,
        namespace: &Namespace,
        fields: Vec<Ident>,
        storage_fields: &[TyStorageField],
        storage_keyword_span: Span,
    ) -> Result<(TyStorageAccess, TypeId), ErrorEmitted> {
        let mut type_checked_buf = vec![];
        let mut fields: Vec<_> = fields.into_iter().rev().collect();

        let first_field = fields.pop().expect("guaranteed by grammar");
        let (ix, initial_field_type) = match storage_fields
            .iter()
            .enumerate()
            .find(|(_, TyStorageField { name, .. })| name == &first_field)
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

        type_checked_buf.push(TyStorageAccessDescriptor {
            name: first_field.clone(),
            type_id: initial_field_type,
            span: first_field.span(),
        });

        let update_struct_decl_and_available_struct_fields = |id: TypeId| match &*type_engine.get(id) {
            TypeInfo::Struct(decl_ref) => {
                let struct_decl = decl_engine.get_struct(decl_ref);
                let fields = struct_decl.fields.clone();

                (Some(struct_decl), fields)
            },
            _ => (None, vec![]),
        };

        // if the previously iterated type was a struct, put its fields here so we know that,
        // in the case of a subfield, we can type check the that the subfield exists and its type.
        let (mut struct_decl, mut available_struct_fields) = update_struct_decl_and_available_struct_fields(initial_field_type);

        // get the initial field's type
        // make sure the next field exists in that type
        for field in fields.into_iter().rev() {
            let decl = struct_decl.expect("If a field is found that means we have the struct declaration.");
            let (struct_can_be_changed, is_public_struct_access) = StructAccessInfo::get_info(&decl, namespace).into();

            match available_struct_fields
                .iter()
                .find(|x| x.name.as_str() == field.as_str())
            {
                Some(struct_field) => {
                    if is_public_struct_access && struct_field.is_private() {
                        return Err(handler.emit_err(CompileError::StructFieldIsPrivate {
                            field_name: (&field).into(),
                            struct_name: decl.call_path.suffix.clone(),
                            field_decl_span: struct_field.name.span(),
                            struct_can_be_changed,
                            usage_context: StructFieldUsageContext::StorageAccess,
                        }));
                    }

                    type_checked_buf.push(TyStorageAccessDescriptor {
                        name: field.clone(),
                        type_id: struct_field.type_argument.type_id,
                        span: field.span().clone(),
                    });
                    (struct_decl, available_struct_fields) =
                        update_struct_decl_and_available_struct_fields(struct_field.type_argument.type_id);
                }
                None => {
                    // Since storage cannot be passed to other modules, the access
                    // is always in the module of the storage declaration.
                    // If the struct cannot be instantiated in this module at all,
                    // we will just show the error, without any additional help lines
                    // showing available fields or anything.
                    // Note that if the struct is empty it can always be instantiated.
                    let struct_can_be_instantiated = !is_public_struct_access || !decl.has_private_fields();

                    let available_fields = if struct_can_be_instantiated {
                        decl.accessible_fields_names(is_public_struct_access)
                    } else {
                        vec![]
                    };

                    return Err(handler.emit_err(CompileError::StructFieldDoesNotExist {
                        field_name: field.into(),
                        available_fields,
                        is_public_struct_access,
                        struct_name: decl.call_path.suffix.clone(),
                        struct_decl_span: decl.span(),
                        struct_is_empty: decl.is_empty(),
                        usage_context: StructFieldUsageContext::StorageAccess,
                    }));
                }
            }
        }

        let return_type = type_checked_buf[type_checked_buf.len() - 1].type_id;

        Ok((
            TyStorageAccess {
                fields: type_checked_buf,
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
