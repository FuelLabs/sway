use sway_error::{
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    language::ty::{self, StructAccessInfo},
    Engines, Namespace, TypeInfo,
};

pub(crate) fn instantiate_struct_field_access(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    parent: ty::TyExpression,
    field_to_access: Ident,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = engines.te();

    let mut current_prefix_te = Box::new(parent);
    let mut current_type = type_engine.get_unaliased(current_prefix_te.return_type);

    let prefix_type_id = current_prefix_te.return_type;
    let prefix_span = current_prefix_te.span.clone();

    // Create the prefix part of the final struct field access expression.
    // This might be an expression that directly evaluates to a struct type,
    // or an arbitrary number of dereferencing expressions where the last one
    // dereferences to a struct type.
    //
    // We will either hit a struct at the end or return an error, so the
    // loop cannot be endless.
    while !current_type.is_struct() {
        match &*current_type {
            TypeInfo::Ref {
                referenced_type, ..
            } => {
                let referenced_type_id = referenced_type.type_id();

                current_prefix_te = Box::new(ty::TyExpression {
                    expression: ty::TyExpressionVariant::Deref(current_prefix_te),
                    return_type: referenced_type_id,
                    span: prefix_span.clone(),
                });

                current_type = type_engine.get_unaliased(referenced_type_id);
            }
            TypeInfo::ErrorRecovery(err) => return Err(*err),
            _ => {
                return Err(handler.emit_err(CompileError::FieldAccessOnNonStruct {
                    actually: engines.help_out(prefix_type_id).to_string(),
                    storage_variable: None,
                    field_name: (&field_to_access).into(),
                    span: prefix_span,
                }))
            }
        };
    }

    let TypeInfo::Struct(struct_decl_ref) = &*current_type else {
        panic!("The current type must be a struct.");
    };

    let decl = engines.de().get_struct(struct_decl_ref);
    let (struct_can_be_changed, is_public_struct_access) =
        StructAccessInfo::get_info(engines, &decl, namespace).into();

    let field = match decl.find_field(&field_to_access) {
        Some(field) => {
            if is_public_struct_access && field.is_private() {
                return Err(handler.emit_err(CompileError::StructFieldIsPrivate {
                    field_name: (&field_to_access).into(),
                    struct_name: decl.call_path.suffix.clone(),
                    field_decl_span: field.name.span(),
                    struct_can_be_changed,
                    usage_context: StructFieldUsageContext::StructFieldAccess,
                }));
            }

            field.clone()
        }
        None => {
            return Err(handler.emit_err(CompileError::StructFieldDoesNotExist {
                field_name: (&field_to_access).into(),
                available_fields: decl.accessible_fields_names(is_public_struct_access),
                is_public_struct_access,
                struct_name: decl.call_path.suffix.clone(),
                struct_decl_span: decl.span(),
                struct_is_empty: decl.is_empty(),
                usage_context: StructFieldUsageContext::StructFieldAccess,
            }));
        }
    };

    let return_type = field.type_argument.type_id();
    Ok(ty::TyExpression {
        expression: ty::TyExpressionVariant::StructFieldAccess {
            resolved_type_of_parent: current_prefix_te.return_type,
            prefix: current_prefix_te,
            field_to_access: field,
            field_instantiation_span: field_to_access.span(),
        },
        return_type,
        span,
    })
}
