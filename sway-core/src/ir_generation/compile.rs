use crate::{
    declaration_engine::declaration_engine::de_get_constant,
    language::{ty, Visibility},
    metadata::MetadataManager,
    semantic_analysis::namespace,
    type_system::{look_up_type_id, LogId, TypeId},
};

use super::{
    const_eval::{compile_const_decl, LookupEnv},
    convert::convert_resolved_typeid,
    function::FnCompiler,
};

use sway_error::error::CompileError;
use sway_ir::{metadata::combine as md_combine, *};
use sway_types::{span::Span, Spanned};

use std::collections::HashMap;

pub(super) fn compile_script(
    context: &mut Context,
    main_function: ty::TyFunctionDeclaration,
    namespace: &namespace::Module,
    declarations: Vec<ty::TyDeclaration>,
    logged_types_map: &HashMap<TypeId, LogId>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Script);
    let mut md_mgr = MetadataManager::default();

    compile_constants(context, &mut md_mgr, module, namespace)?;
    compile_declarations(context, &mut md_mgr, module, namespace, declarations)?;
    compile_function(
        context,
        &mut md_mgr,
        module,
        main_function,
        logged_types_map,
    )?;

    Ok(module)
}

pub(super) fn compile_predicate(
    context: &mut Context,
    main_function: ty::TyFunctionDeclaration,
    namespace: &namespace::Module,
    declarations: Vec<ty::TyDeclaration>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Predicate);
    let mut md_mgr = MetadataManager::default();

    compile_constants(context, &mut md_mgr, module, namespace)?;
    compile_declarations(context, &mut md_mgr, module, namespace, declarations)?;
    compile_function(context, &mut md_mgr, module, main_function, &HashMap::new())?;

    Ok(module)
}

pub(super) fn compile_contract(
    context: &mut Context,
    abi_entries: Vec<ty::TyFunctionDeclaration>,
    namespace: &namespace::Module,
    declarations: Vec<ty::TyDeclaration>,
    logged_types_map: &HashMap<TypeId, LogId>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Contract);
    let mut md_mgr = MetadataManager::default();

    compile_constants(context, &mut md_mgr, module, namespace)?;
    compile_declarations(context, &mut md_mgr, module, namespace, declarations)?;
    for decl in abi_entries {
        compile_abi_method(context, &mut md_mgr, module, decl, logged_types_map)?;
    }

    Ok(module)
}

pub(crate) fn compile_constants(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: &namespace::Module,
) -> Result<(), CompileError> {
    for decl_name in module_ns.get_all_declared_symbols() {
        compile_const_decl(
            &mut LookupEnv {
                context,
                md_mgr,
                module,
                module_ns: Some(module_ns),
                lookup: compile_const_decl,
            },
            decl_name,
        )?;
    }

    for submodule_ns in module_ns.submodules().values() {
        compile_constants(context, md_mgr, module, submodule_ns)?;
    }

    Ok(())
}

// We don't really need to compile these declarations other than `const`s since:
// a) function decls are inlined into their call site and can be (re)created there, though ideally
//    we'd give them their proper name by compiling them here.
// b) struct decls are also inlined at their instantiation site.
// c) ditto for enums.
//
// And for structs and enums in particular, we must ignore those with embedded generic types as
// they are monomorphised only at the instantation site.  We must ignore the generic declarations
// altogether anyway.
fn compile_declarations(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    namespace: &namespace::Module,
    declarations: Vec<ty::TyDeclaration>,
) -> Result<(), CompileError> {
    for declaration in declarations {
        match declaration {
            ty::TyDeclaration::ConstantDeclaration(ref decl_id) => {
                let decl = de_get_constant(decl_id.clone(), &declaration.span())?;
                compile_const_decl(
                    &mut LookupEnv {
                        context,
                        md_mgr,
                        module,
                        module_ns: Some(namespace),
                        lookup: compile_const_decl,
                    },
                    &decl.name,
                )?;
            }

            ty::TyDeclaration::FunctionDeclaration(_decl) => {
                // We no longer compile functions other than `main()` until we can improve the name
                // resolution.  Currently there isn't enough information in the AST to fully
                // distinguish similarly named functions and especially trait methods.
                //
                //compile_function(context, module, decl).map(|_| ())?
            }
            ty::TyDeclaration::ImplTrait(_) => {
                // And for the same reason we don't need to compile impls at all.
                //
                // compile_impl(
                //    context,
                //    module,
                //    type_implementing_for,
                //    methods,
                //)?,
            }

            ty::TyDeclaration::StructDeclaration(_)
            | ty::TyDeclaration::EnumDeclaration(_)
            | ty::TyDeclaration::TraitDeclaration(_)
            | ty::TyDeclaration::VariableDeclaration(_)
            | ty::TyDeclaration::AbiDeclaration(_)
            | ty::TyDeclaration::GenericTypeForFunctionScope { .. }
            | ty::TyDeclaration::StorageDeclaration(_)
            | ty::TyDeclaration::ErrorRecovery => (),
        }
    }
    Ok(())
}

pub(super) fn compile_function(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: ty::TyFunctionDeclaration,
    logged_types_map: &HashMap<TypeId, LogId>,
) -> Result<Option<Function>, CompileError> {
    // Currently monomorphisation of generics is inlined into main() and the functions with generic
    // args are still present in the AST declarations, but they can be ignored.
    if !ast_fn_decl.type_parameters.is_empty() {
        Ok(None)
    } else {
        let args = ast_fn_decl
            .parameters
            .iter()
            .map(|param| convert_fn_param(context, param))
            .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

        compile_fn_with_args(
            context,
            md_mgr,
            module,
            ast_fn_decl,
            args,
            None,
            logged_types_map,
        )
        .map(&Some)
    }
}

fn convert_fn_param(
    context: &mut Context,
    param: &ty::TyFunctionParameter,
) -> Result<(String, Type, Span), CompileError> {
    convert_resolved_typeid(context, &param.type_id, &param.type_span).map(|ty| {
        (
            param.name.as_str().into(),
            if param.is_reference && look_up_type_id(param.type_id).is_copy_type() {
                Type::Pointer(Pointer::new(context, ty, param.is_mutable, None))
            } else {
                ty
            },
            param.name.span(),
        )
    })
}

fn compile_fn_with_args(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: ty::TyFunctionDeclaration,
    args: Vec<(String, Type, Span)>,
    selector: Option<[u8; 4]>,
    logged_types_map: &HashMap<TypeId, LogId>,
) -> Result<Function, CompileError> {
    let inline_opt = ast_fn_decl.inline();
    let ty::TyFunctionDeclaration {
        name,
        body,
        return_type,
        return_type_span,
        visibility,
        purity,
        span,
        ..
    } = ast_fn_decl;

    let mut args = args
        .into_iter()
        .map(|(name, ty, span)| (name, ty, md_mgr.span_to_md(context, &span)))
        .collect::<Vec<_>>();

    let ret_type = convert_resolved_typeid(context, &return_type, &return_type_span)?;

    let is_entry = selector.is_some()
        || (matches!(module.get_kind(context), Kind::Script | Kind::Predicate)
            && name.as_str() == "main");
    let returns_by_ref = !is_entry && !ret_type.is_copy_type();
    if returns_by_ref {
        // Instead of 'returning' a by-ref value we make the last argument an 'out' parameter.
        args.push((
            "__ret_value".to_owned(),
            Type::Pointer(Pointer::new(context, ret_type, true, None)),
            md_mgr.span_to_md(context, &return_type_span),
        ));
    }

    let span_md_idx = md_mgr.span_to_md(context, &span);
    let storage_md_idx = md_mgr.purity_to_md(context, purity);
    let mut metadata = md_combine(context, &span_md_idx, &storage_md_idx);

    if let Some(inline) = inline_opt {
        let inline_md_idx = md_mgr.inline_to_md(context, inline);
        metadata = md_combine(context, &metadata, &inline_md_idx);
    }

    let func = Function::new(
        context,
        module,
        name.as_str().to_owned(),
        args,
        ret_type,
        selector,
        visibility == Visibility::Public,
        metadata,
    );

    let mut compiler = FnCompiler::new(context, module, func, returns_by_ref, logged_types_map);
    let mut ret_val = compiler.compile_code_block(context, md_mgr, body)?;

    // Special case: sometimes the returned value at the end of the function block is hacked
    // together and is invalid.  This can happen with diverging control flow or with implicit
    // returns.  We can double check here and make sure the return value type is correct.
    ret_val = match ret_val.get_type(context) {
        Some(ret_val_type) if ret_type.eq(context, &ret_val_type.strip_ptr_type(context)) => {
            ret_val
        }

        // Mismatched or unavailable type.  Set ret_val to a correctly typed Undef.
        _otherwise => Value::new_constant(context, Constant::get_undef(ret_type)),
    };

    // Another special case: if the last expression in a function is a return then we don't want to
    // add another implicit return instruction here, as `ret_val` will be unit regardless of the
    // function return type actually is.  This new RET will be going into an unreachable block
    // which is valid, but pointless and we should avoid it due to the aforementioned potential
    // type conflict.
    //
    // To tell if this is the case we can check that the current block is empty and has no
    // predecessors (and isn't the entry block which has none by definition), implying the most
    // recent instruction was a RET.
    let already_returns = compiler
        .current_block
        .is_terminated_by_ret_or_revert(context);
    if !already_returns
        && (compiler.current_block.num_instructions(context) > 0
            || compiler.current_block == compiler.function.get_entry_block(context)
            || compiler.current_block.num_predecessors(context) > 0)
    {
        if returns_by_ref {
            // Need to copy ref-type return values to the 'out' parameter.
            ret_val = compiler.compile_copy_to_last_arg(context, ret_val, None);
        }
        if ret_type.eq(context, &Type::Unit) {
            ret_val = Constant::get_unit(context);
        }
        compiler.current_block.ins(context).ret(ret_val, ret_type);
    }
    Ok(func)
}

/* Disabled until we can improve symbol resolution.  See comments above in compile_declarations().

fn compile_impl(
    context: &mut Context,
    module: Module,
    self_type: TypeInfo,
    ast_methods: Vec<TypedFunctionDeclaration>,
) -> Result<(), CompileError> {
    for method in ast_methods {
        let args = method
            .parameters
            .iter()
            .map(|param| {
                if param.name.as_str() == "self" {
                    convert_resolved_type(context, &self_type)
                } else {
                    convert_resolved_typeid(context, &param.type_id, &param.type_span)
                }
                .map(|ty| (param.name.as_str().into(), ty, param.name.span().clone()))
            })
            .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

        compile_fn_with_args(context, module, method, args, None)?;
    }
    Ok(())
}
*/

fn compile_abi_method(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: ty::TyFunctionDeclaration,
    logged_types_map: &HashMap<TypeId, LogId>,
) -> Result<Function, CompileError> {
    // Use the error from .to_fn_selector_value() if possible, else make an CompileError::Internal.
    let get_selector_result = ast_fn_decl.to_fn_selector_value();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let selector = match get_selector_result.ok(&mut warnings, &mut errors) {
        Some(selector) => selector,
        None => {
            return if !errors.is_empty() {
                Err(errors[0].clone())
            } else {
                Err(CompileError::InternalOwned(
                    format!(
                        "Cannot generate selector for ABI method: {}",
                        ast_fn_decl.name.as_str()
                    ),
                    ast_fn_decl.name.span(),
                ))
            };
        }
    };

    let args = ast_fn_decl
        .parameters
        .iter()
        .map(|param| {
            convert_resolved_typeid(context, &param.type_id, &param.type_span)
                .map(|ty| (param.name.as_str().into(), ty, param.name.span()))
        })
        .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

    compile_fn_with_args(
        context,
        md_mgr,
        module,
        ast_fn_decl,
        args,
        Some(selector),
        logged_types_map,
    )
}
