use crate::{
    error::CompileError,
    parse_tree::{Purity, Visibility},
    semantic_analysis::{ast_node::*, namespace},
};

use super::{
    const_eval::{compile_const_decl, LookupEnv},
    convert::convert_resolved_typeid,
    function::FnCompiler,
};

use sway_ir::*;
use sway_types::{span::Span, Spanned};

pub(super) fn compile_script(
    context: &mut Context,
    main_function: TypedFunctionDeclaration,
    namespace: &namespace::Module,
    declarations: Vec<TypedDeclaration>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Script);

    compile_constants(context, module, namespace, false)?;
    compile_declarations(context, module, namespace, declarations)?;
    compile_function(context, module, main_function)?;

    Ok(module)
}

pub(super) fn compile_contract(
    context: &mut Context,
    abi_entries: Vec<TypedFunctionDeclaration>,
    namespace: &namespace::Module,
    declarations: Vec<TypedDeclaration>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Contract);

    compile_constants(context, module, namespace, false)?;
    compile_declarations(context, module, namespace, declarations)?;
    for decl in abi_entries {
        compile_abi_method(context, module, decl)?;
    }

    Ok(module)
}

fn compile_constants(
    context: &mut Context,
    module: Module,
    module_ns: &namespace::Module,
    public_only: bool,
) -> Result<(), CompileError> {
    for decl_name in module_ns.get_all_declared_symbols() {
        compile_const_decl(
            &mut LookupEnv {
                context,
                module,
                module_ns: Some(module_ns),
                public_only,
                lookup: compile_const_decl,
            },
            decl_name,
        )?;
    }

    for submodule_ns in module_ns.submodules().values() {
        compile_constants(context, module, submodule_ns, true)?;
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
    module: Module,
    namespace: &namespace::Module,
    declarations: Vec<TypedDeclaration>,
) -> Result<(), CompileError> {
    for declaration in declarations {
        match declaration {
            TypedDeclaration::ConstantDeclaration(decl) => {
                compile_const_decl(
                    &mut LookupEnv {
                        context,
                        module,
                        module_ns: Some(namespace),
                        public_only: false,
                        lookup: compile_const_decl,
                    },
                    &decl.name,
                )?;
            }

            TypedDeclaration::FunctionDeclaration(_decl) => {
                // We no longer compile functions other than `main()` until we can improve the name
                // resolution.  Currently there isn't enough information in the AST to fully
                // distinguish similarly named functions and especially trait methods.
                //
                //compile_function(context, module, decl).map(|_| ())?
            }
            TypedDeclaration::ImplTrait(_) => {
                // And for the same reason we don't need to compile impls at all.
                //
                // compile_impl(
                //    context,
                //    module,
                //    type_implementing_for,
                //    methods,
                //)?,
            }

            TypedDeclaration::StructDeclaration(_)
            | TypedDeclaration::EnumDeclaration(_)
            | TypedDeclaration::TraitDeclaration(_)
            | TypedDeclaration::VariableDeclaration(_)
            | TypedDeclaration::Reassignment(_)
            | TypedDeclaration::StorageReassignment(_)
            | TypedDeclaration::AbiDeclaration(_)
            | TypedDeclaration::GenericTypeForFunctionScope { .. }
            | TypedDeclaration::StorageDeclaration(_)
            | TypedDeclaration::ErrorRecovery
            | TypedDeclaration::Break { .. }
            | TypedDeclaration::Continue { .. } => (),
        }
    }
    Ok(())
}

pub(super) fn compile_function(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
) -> Result<Option<Function>, CompileError> {
    // Currently monomorphisation of generics is inlined into main() and the functions with generic
    // args are still present in the AST declarations, but they can be ignored.
    if !ast_fn_decl.type_parameters.is_empty() {
        Ok(None)
    } else {
        let args = ast_fn_decl
            .parameters
            .iter()
            .map(|param| {
                convert_resolved_typeid(context, &param.type_id, &param.type_span)
                    .map(|ty| (param.name.as_str().into(), ty, param.name.span()))
            })
            .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

        compile_fn_with_args(context, module, ast_fn_decl, args, None).map(&Some)
    }
}

fn compile_fn_with_args(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
    args: Vec<(String, Type, Span)>,
    selector: Option<[u8; 4]>,
) -> Result<Function, CompileError> {
    let TypedFunctionDeclaration {
        name,
        body,
        return_type,
        return_type_span,
        visibility,
        purity,
        span,
        ..
    } = ast_fn_decl;

    let args = args
        .into_iter()
        .map(|(name, ty, span)| (name, ty, MetadataIndex::from_span(context, &span)))
        .collect();
    let ret_type = convert_resolved_typeid(context, &return_type, &return_type_span)?;
    let span_md_idx = MetadataIndex::from_span(context, &span);
    let storage_md_idx = if purity == Purity::Pure {
        None
    } else {
        Some(MetadataIndex::get_storage_index(
            context,
            match purity {
                Purity::Pure => unreachable!("Already checked for Pure above."),
                Purity::Reads => StorageOperation::Reads,
                Purity::Writes => StorageOperation::Writes,
                Purity::ReadsWrites => StorageOperation::ReadsWrites,
            },
        ))
    };
    let func = Function::new(
        context,
        module,
        name.as_str().to_owned(),
        args,
        ret_type,
        selector,
        visibility == Visibility::Public,
        span_md_idx,
        storage_md_idx,
    );

    // We clone the struct symbols here, as they contain the globals; any new local declarations
    // may remain within the function scope.
    let mut compiler = FnCompiler::new(context, module, func);

    let mut ret_val = compiler.compile_code_block(context, body)?;

    // Special case: if the return type is unit but the return value type is not, then we have an
    // implicit return from the last expression in the code block having a semi-colon.  This isn't
    // codified in the AST explicitly so we need to make a unit to return here.
    if ret_type.eq(context, &Type::Unit) && !matches!(ret_val.get_type(context), Some(Type::Unit)) {
        // NOTE: We're replacing the ret_val and throwing away whatever it used to be, as it is
        // never actually used anyway.
        ret_val = Constant::get_unit(context, None);
    }

    let already_returns = compiler.current_block.is_terminated_by_ret(context);

    // Another special case: if the last expression in a function is a return then we don't want to
    // add another implicit return instruction here, as `ret_val` will be unit regardless of the
    // function return type actually is.  This new RET will be going into an unreachable block
    // which is valid, but pointless and we should avoid it due to the aforementioned potential
    // type conflict.
    //
    // To tell if this is the case we can check that the current block is empty and has no
    // predecessors (and isn't the entry block which has none by definition), implying the most
    // recent instruction was a RET.
    if !already_returns
        && (compiler.current_block.num_instructions(context) > 0
            || compiler.current_block == compiler.function.get_entry_block(context)
            || compiler.current_block.num_predecessors(context) > 0)
    {
        if ret_type.eq(context, &Type::Unit) {
            ret_val = Constant::get_unit(context, None);
        }
        compiler
            .current_block
            .ins(context)
            .ret(ret_val, ret_type, None);
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
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
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

    compile_fn_with_args(context, module, ast_fn_decl, args, Some(selector))
}
