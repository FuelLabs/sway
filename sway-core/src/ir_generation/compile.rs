use crate::{
    decl_engine::{DeclId, DeclRef},
    language::{ty, Visibility},
    metadata::MetadataManager,
    semantic_analysis::namespace,
    type_system::{LogId, MessageId, TypeId},
    Engines, TypeEngine,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_script(
    engines: Engines<'_>,
    context: &mut Context,
    main_function: &ty::TyFunctionDeclaration,
    namespace: &namespace::Module,
    declarations: &[ty::TyDeclaration],
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    test_fns: &[(ty::TyFunctionDeclaration, DeclRef)],
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Script);
    let mut md_mgr = MetadataManager::default();

    compile_constants(engines, context, &mut md_mgr, module, namespace)?;
    compile_declarations(
        engines,
        context,
        &mut md_mgr,
        module,
        namespace,
        declarations,
    )?;
    compile_entry_function(
        engines,
        context,
        &mut md_mgr,
        module,
        main_function,
        logged_types_map,
        messages_types_map,
        None,
    )?;
    compile_tests(
        engines,
        context,
        &mut md_mgr,
        module,
        logged_types_map,
        messages_types_map,
        test_fns,
    )?;

    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_predicate(
    engines: Engines<'_>,
    context: &mut Context,
    main_function: &ty::TyFunctionDeclaration,
    namespace: &namespace::Module,
    declarations: &[ty::TyDeclaration],
    logged_types: &HashMap<TypeId, LogId>,
    messages_types: &HashMap<TypeId, MessageId>,
    test_fns: &[(ty::TyFunctionDeclaration, DeclRef)],
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Predicate);
    let mut md_mgr = MetadataManager::default();

    compile_constants(engines, context, &mut md_mgr, module, namespace)?;
    compile_declarations(
        engines,
        context,
        &mut md_mgr,
        module,
        namespace,
        declarations,
    )?;
    compile_entry_function(
        engines,
        context,
        &mut md_mgr,
        module,
        main_function,
        &HashMap::new(),
        &HashMap::new(),
        None,
    )?;
    compile_tests(
        engines,
        context,
        &mut md_mgr,
        module,
        logged_types,
        messages_types,
        test_fns,
    )?;

    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_contract(
    context: &mut Context,
    abi_entries: &[ty::TyFunctionDeclaration],
    namespace: &namespace::Module,
    declarations: &[ty::TyDeclaration],
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    test_fns: &[(ty::TyFunctionDeclaration, DeclRef)],
    engines: Engines<'_>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Contract);
    let mut md_mgr = MetadataManager::default();

    compile_constants(engines, context, &mut md_mgr, module, namespace)?;
    compile_declarations(
        engines,
        context,
        &mut md_mgr,
        module,
        namespace,
        declarations,
    )?;
    for decl in abi_entries {
        compile_abi_method(
            context,
            &mut md_mgr,
            module,
            decl,
            logged_types_map,
            messages_types_map,
            engines,
        )?;
    }
    compile_tests(
        engines,
        context,
        &mut md_mgr,
        module,
        logged_types_map,
        messages_types_map,
        test_fns,
    )?;

    Ok(module)
}

pub(super) fn compile_library(
    engines: Engines<'_>,
    context: &mut Context,
    namespace: &namespace::Module,
    declarations: &[ty::TyDeclaration],
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    test_fns: &[(ty::TyFunctionDeclaration, DeclRef)],
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Library);
    let mut md_mgr = MetadataManager::default();

    compile_constants(engines, context, &mut md_mgr, module, namespace)?;
    compile_declarations(
        engines,
        context,
        &mut md_mgr,
        module,
        namespace,
        declarations,
    )?;
    compile_tests(
        engines,
        context,
        &mut md_mgr,
        module,
        logged_types_map,
        messages_types_map,
        test_fns,
    )?;

    Ok(module)
}

pub(crate) fn compile_constants(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: &namespace::Module,
) -> Result<(), CompileError> {
    let (type_engine, decl_engine) = engines.unwrap();
    for decl_name in module_ns.get_all_declared_symbols() {
        compile_const_decl(
            &mut LookupEnv {
                type_engine,
                decl_engine,
                context,
                md_mgr,
                module,
                module_ns: Some(module_ns),
                function_compiler: None,
                lookup: compile_const_decl,
            },
            decl_name,
        )?;
    }

    for submodule_ns in module_ns.submodules().values() {
        compile_constants(engines, context, md_mgr, module, submodule_ns)?;
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
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    namespace: &namespace::Module,
    declarations: &[ty::TyDeclaration],
) -> Result<(), CompileError> {
    let (type_engine, decl_engine) = engines.unwrap();
    for declaration in declarations {
        match declaration {
            ty::TyDeclaration::ConstantDeclaration {
                decl_id, decl_span, ..
            } => {
                let decl = decl_engine.get_constant(decl_id, decl_span)?;
                compile_const_decl(
                    &mut LookupEnv {
                        type_engine,
                        decl_engine,
                        context,
                        md_mgr,
                        module,
                        module_ns: Some(namespace),
                        function_compiler: None,
                        lookup: compile_const_decl,
                    },
                    &decl.name,
                )?;
            }

            ty::TyDeclaration::FunctionDeclaration { .. } => {
                // We no longer compile functions other than `main()` until we can improve the name
                // resolution.  Currently there isn't enough information in the AST to fully
                // distinguish similarly named functions and especially trait methods.
                //
                //compile_function(context, module, decl).map(|_| ())?
            }
            ty::TyDeclaration::ImplTrait { .. } => {
                // And for the same reason we don't need to compile impls at all.
                //
                // compile_impl(
                //    context,
                //    module,
                //    type_implementing_for,
                //    methods,
                //)?,
            }

            ty::TyDeclaration::StructDeclaration { .. }
            | ty::TyDeclaration::EnumDeclaration { .. }
            | ty::TyDeclaration::TraitDeclaration { .. }
            | ty::TyDeclaration::VariableDeclaration(_)
            | ty::TyDeclaration::AbiDeclaration { .. }
            | ty::TyDeclaration::GenericTypeForFunctionScope { .. }
            | ty::TyDeclaration::StorageDeclaration { .. }
            | ty::TyDeclaration::ErrorRecovery(_) => (),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_function(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &ty::TyFunctionDeclaration,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    is_entry: bool,
    test_decl_ref: Option<DeclRef>,
) -> Result<Option<Function>, CompileError> {
    let type_engine = engines.te();
    // Currently monomorphization of generics is inlined into main() and the functions with generic
    // args are still present in the AST declarations, but they can be ignored.
    if !ast_fn_decl.type_parameters.is_empty_excluding_self() {
        Ok(None)
    } else {
        let args = ast_fn_decl
            .parameters
            .iter()
            .map(|param| convert_fn_param(type_engine, context, param))
            .collect::<Result<Vec<(String, Type, bool, Span)>, CompileError>>()?;

        compile_fn_with_args(
            engines,
            context,
            md_mgr,
            module,
            ast_fn_decl,
            is_entry,
            args,
            None,
            logged_types_map,
            messages_types_map,
            test_decl_ref,
        )
        .map(Some)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_entry_function(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &ty::TyFunctionDeclaration,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    test_decl_ref: Option<DeclRef>,
) -> Result<Function, CompileError> {
    let is_entry = true;
    compile_function(
        engines,
        context,
        md_mgr,
        module,
        ast_fn_decl,
        logged_types_map,
        messages_types_map,
        is_entry,
        test_decl_ref,
    )
    .map(|f| f.expect("entry point should never contain generics"))
}

pub(super) fn compile_tests(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    test_fns: &[(ty::TyFunctionDeclaration, DeclRef)],
) -> Result<Vec<Function>, CompileError> {
    test_fns
        .iter()
        .map(|(ast_fn_decl, decl_ref)| {
            compile_entry_function(
                engines,
                context,
                md_mgr,
                module,
                ast_fn_decl,
                logged_types_map,
                messages_types_map,
                Some(decl_ref.clone()),
            )
        })
        .collect()
}

fn convert_fn_param(
    type_engine: &TypeEngine,
    context: &mut Context,
    param: &ty::TyFunctionParameter,
) -> Result<(String, Type, bool, Span), CompileError> {
    convert_resolved_typeid(
        type_engine,
        context,
        &param.type_argument.type_id,
        &param.type_argument.span,
    )
    .map(|ty| {
        let by_ref =
            param.is_reference && type_engine.get(param.type_argument.type_id).is_copy_type();
        (param.name.as_str().into(), ty, by_ref, param.name.span())
    })
}

#[allow(clippy::too_many_arguments)]
fn compile_fn_with_args(
    engines: Engines<'_>,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &ty::TyFunctionDeclaration,
    is_entry: bool,
    args: Vec<(String, Type, bool, Span)>,
    selector: Option<[u8; 4]>,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    test_decl_ref: Option<DeclRef>,
) -> Result<Function, CompileError> {
    let type_engine = engines.te();

    let inline_opt = ast_fn_decl.inline();
    let ty::TyFunctionDeclaration {
        name,
        body,
        return_type,
        visibility,
        purity,
        span,
        ..
    } = ast_fn_decl;

    let mut args = args
        .into_iter()
        .map(|(name, ty, by_ref, span)| (name, ty, by_ref, md_mgr.span_to_md(context, &span)))
        .collect::<Vec<_>>();

    let ret_type = convert_resolved_typeid(
        type_engine,
        context,
        &return_type.type_id,
        &return_type.span,
    )?;

    let returns_by_ref = !is_entry && !type_engine.get(return_type.type_id).is_copy_type();
    if returns_by_ref {
        // Instead of 'returning' a by-ref value we make the last argument an 'out' parameter.
        args.push((
            "__ret_value".to_owned(),
            ret_type,
            true,
            md_mgr.span_to_md(context, &return_type.span),
        ));
    }

    let span_md_idx = md_mgr.span_to_md(context, span);
    let storage_md_idx = md_mgr.purity_to_md(context, *purity);
    let mut metadata = md_combine(context, &span_md_idx, &storage_md_idx);

    let decl_index = test_decl_ref.map(|decl_ref| *DeclId::from(&decl_ref));
    if let Some(decl_index) = decl_index {
        let test_decl_index_md_idx = md_mgr.test_decl_index_to_md(context, decl_index);
        metadata = md_combine(context, &metadata, &test_decl_index_md_idx);
    }
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
        *visibility == Visibility::Public,
        is_entry,
        metadata,
    );

    let mut compiler = FnCompiler::new(
        engines,
        context,
        module,
        func,
        returns_by_ref,
        logged_types_map,
        messages_types_map,
    );
    let mut ret_val = compiler.compile_code_block(context, md_mgr, body)?;

    // Special case: sometimes the returned value at the end of the function block is hacked
    // together and is invalid.  This can happen with diverging control flow or with implicit
    // returns.  We can double check here and make sure the return value type is correct.
    ret_val = match ret_val.get_type(context) {
        Some(ret_val_type) if ret_type.eq(context, &ret_val_type) => ret_val,

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
        if ret_type.is_unit(context) {
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
    ast_fn_decl: &ty::TyFunctionDeclaration,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    engines: Engines<'_>,
) -> Result<Function, CompileError> {
    let type_engine = engines.te();

    // Use the error from .to_fn_selector_value() if possible, else make an CompileError::Internal.
    let get_selector_result = ast_fn_decl.to_fn_selector_value(type_engine);
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

    // An ABI method is always an entry point.
    let is_entry = true;

    let args = ast_fn_decl
        .parameters
        .iter()
        .map(|param| {
            convert_resolved_typeid(
                type_engine,
                context,
                &param.type_argument.type_id,
                &param.type_argument.span,
            )
            .map(|ty| (param.name.as_str().into(), ty, false, param.name.span()))
        })
        .collect::<Result<Vec<(String, Type, bool, Span)>, CompileError>>()?;

    compile_fn_with_args(
        engines,
        context,
        md_mgr,
        module,
        ast_fn_decl,
        is_entry,
        args,
        Some(selector),
        logged_types_map,
        messages_types_map,
        None,
    )
}
