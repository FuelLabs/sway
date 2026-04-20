use super::{
    const_eval::{compile_const_decl, LookupEnv},
    convert::convert_resolved_type_id,
    function::FnCompiler,
    CompiledFunctionCache,
};
use crate::{
    decl_engine::{DeclEngineGet, DeclId, DeclRefFunction},
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, KeyedTyFunctionDecl,
        PanickingFunctionCache,
    },
    language::{
        ty::{self, TyExpression},
        Visibility,
    },
    metadata::MetadataManager,
    namespace::ResolvedDeclaration,
    semantic_analysis::namespace,
    type_system::TypeId,
    types::{LogId, MessageId},
    Engines, PanicOccurrences, PanickingCallOccurrences, TypeInfo,
};
use std::{
    cell::Cell,
    collections::{BTreeSet, HashMap},
    sync::Arc,
};
use sway_error::{
    error::{CompileError, TrivialCheckFailedData},
    handler::Handler,
};
use sway_ir::{metadata::combine as md_combine, *};
use sway_types::{integer_bits::IntegerBits, Ident, Named, ProgramId, Span, Spanned};

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_script(
    engines: &Engines,
    ctx: &mut Context,
    entry_function: &DeclId<ty::TyFunctionDecl>,
    namespace: &namespace::Namespace,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_fns: &[(Arc<ty::TyFunctionDecl>, DeclRefFunction)],
    compiled_fn_cache: &mut CompiledFunctionCache,
    md_mgr: &mut MetadataManager,
    module: Module,
) -> Result<Module, Vec<CompileError>> {
    compile_constants_for_package(engines, ctx, module, namespace)?;

    compile_configurables(
        engines,
        ctx,
        md_mgr,
        module,
        namespace.current_package_root_module(),
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        compiled_fn_cache,
    )
    .map_err(|err| vec![err])?;
    compile_entry_function(
        engines,
        ctx,
        md_mgr,
        module,
        entry_function,
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        None,
        compiled_fn_cache,
    )?;
    compile_tests(
        engines,
        ctx,
        md_mgr,
        module,
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        test_fns,
        compiled_fn_cache,
    )?;

    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_predicate(
    engines: &Engines,
    context: &mut Context,
    entry_function: &DeclId<ty::TyFunctionDecl>,
    namespace: &namespace::Namespace,
    logged_types: &HashMap<TypeId, LogId>,
    messages_types: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_fns: &[(Arc<ty::TyFunctionDecl>, DeclRefFunction)],
    compiled_fn_cache: &mut CompiledFunctionCache,
    md_mgr: &mut MetadataManager,
    module: Module,
) -> Result<Module, Vec<CompileError>> {
    compile_constants_for_package(engines, context, module, namespace)?;
    compile_configurables(
        engines,
        context,
        md_mgr,
        module,
        namespace.current_package_root_module(),
        logged_types,
        messages_types,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        compiled_fn_cache,
    )
    .map_err(|err| vec![err])?;
    compile_entry_function(
        engines,
        context,
        md_mgr,
        module,
        entry_function,
        &HashMap::new(),
        &HashMap::new(),
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        None,
        compiled_fn_cache,
    )?;
    compile_tests(
        engines,
        context,
        md_mgr,
        module,
        logged_types,
        messages_types,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        test_fns,
        compiled_fn_cache,
    )?;

    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_contract(
    context: &mut Context,
    entry_function: Option<&DeclId<ty::TyFunctionDecl>>,
    abi_entries: &[DeclId<ty::TyFunctionDecl>],
    namespace: &namespace::Namespace,
    declarations: &[ty::TyDecl],
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_fns: &[(Arc<ty::TyFunctionDecl>, DeclRefFunction)],
    engines: &Engines,
    compiled_fn_cache: &mut CompiledFunctionCache,
    md_mgr: &mut MetadataManager,
    module: Module,
) -> Result<Module, Vec<CompileError>> {
    compile_constants_for_package(engines, context, module, namespace)?;
    compile_configurables(
        engines,
        context,
        md_mgr,
        module,
        namespace.current_package_root_module(),
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        compiled_fn_cache,
    )
    .map_err(|err| vec![err])?;

    // In the case of the new encoding, we need to compile only the entry function.
    // The compilation of the entry function will recursively compile all the
    // ABI methods and the fallback function, if specified.
    if context.experimental.new_encoding {
        let Some(entry_function) = entry_function else {
            return Err(vec![CompileError::Internal(
                "Entry function not specified when compiling contract with new encoding.",
                Span::dummy(),
            )]);
        };
        compile_entry_function(
            engines,
            context,
            md_mgr,
            module,
            entry_function,
            logged_types_map,
            messages_types_map,
            panic_occurrences,
            panicking_call_occurrences,
            panicking_fn_cache,
            None,
            compiled_fn_cache,
        )?;
    } else {
        // In the case of the encoding v0, we need to compile individual ABI entries
        // and the fallback function.
        for decl in abi_entries {
            compile_encoding_v0_abi_method(
                context,
                md_mgr,
                module,
                decl,
                logged_types_map,
                messages_types_map,
                panic_occurrences,
                panicking_call_occurrences,
                panicking_fn_cache,
                engines,
                compiled_fn_cache,
            )?;
        }

        for decl in declarations {
            if let ty::TyDecl::FunctionDecl(decl) = decl {
                let decl_id = decl.decl_id;
                let decl = engines.de().get(&decl_id);
                if decl.is_fallback() {
                    compile_encoding_v0_abi_method(
                        context,
                        md_mgr,
                        module,
                        &decl_id,
                        logged_types_map,
                        messages_types_map,
                        panic_occurrences,
                        panicking_call_occurrences,
                        panicking_fn_cache,
                        engines,
                        compiled_fn_cache,
                    )?;
                }
            }
        }
    }

    compile_tests(
        engines,
        context,
        md_mgr,
        module,
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        test_fns,
        compiled_fn_cache,
    )?;

    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_library(
    engines: &Engines,
    ctx: &mut Context,
    namespace: &namespace::Namespace,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_fns: &[(Arc<ty::TyFunctionDecl>, DeclRefFunction)],
    compiled_fn_cache: &mut CompiledFunctionCache,
    md_mgr: &mut MetadataManager,
    module: Module,
) -> Result<Module, Vec<CompileError>> {
    compile_constants_for_package(engines, ctx, module, namespace)?;
    compile_tests(
        engines,
        ctx,
        md_mgr,
        module,
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        test_fns,
        compiled_fn_cache,
    )?;

    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_constants_for_package(
    engines: &Engines,
    context: &mut Context,
    module: Module,
    namespace: &namespace::Namespace,
) -> Result<Module, Vec<CompileError>> {
    // Traverses the tree of externals and collects all constants
    fn traverse(
        engines: &Engines,
        context: &mut Context,
        module: Module,
        current: &namespace::Package,
    ) -> Result<Module, Vec<CompileError>> {
        let mut md_mgr = MetadataManager::default();

        // Collect constant for all dependencies
        for ext_package in current.external_packages.values() {
            traverse(engines, context, module, ext_package)?;
        }

        compile_constants(engines, context, &mut md_mgr, module, current.root_module())
            .map_err(|err| vec![err])?;

        Ok(module)
    }

    traverse(engines, context, module, namespace.current_package_ref())
}

fn compile_constants(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: &namespace::Module,
) -> Result<(), CompileError> {
    for decl_name in module_ns.root_items().get_all_declared_symbols() {
        if let Some(resolved_decl) = module_ns.root_items().symbols.get(decl_name) {
            if let ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) =
                &resolved_decl.expect_typed_ref()
            {
                let const_decl = engines.de().get_constant(decl_id);
                let call_path = const_decl.call_path.clone();
                compile_const_decl(
                    &mut LookupEnv {
                        engines,
                        context,
                        md_mgr,
                        module,
                        module_ns: Some(module_ns),
                        function_compiler: None,
                        lookup: compile_const_decl,
                    },
                    &call_path,
                    &Some((*const_decl).clone()),
                )?;
            }
        }
    }

    for submodule_ns in module_ns.submodules().values() {
        compile_constants(engines, context, md_mgr, module, submodule_ns)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_configurables(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: &namespace::Module,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    compiled_fn_cache: &mut CompiledFunctionCache,
) -> Result<(), CompileError> {
    for decl_name in module_ns.root_items().get_all_declared_symbols() {
        if let Some(ResolvedDeclaration::Typed(ty::TyDecl::ConfigurableDecl(
            ty::ConfigurableDecl { decl_id, .. },
        ))) = module_ns.root_items().symbols.get(decl_name)
        {
            let decl = engines.de().get(decl_id);

            let ty = convert_resolved_type_id(
                engines,
                context,
                md_mgr,
                module,
                None,
                decl.type_ascription.type_id,
                &decl.type_ascription.span(),
            )
            .unwrap();
            let ptr_ty = Type::new_typed_pointer(context, ty);

            let constant = super::const_eval::compile_constant_expression_to_constant(
                engines,
                context,
                md_mgr,
                module,
                Some(module_ns),
                None,
                decl.value.as_ref().unwrap(),
            )?;

            let opt_metadata = md_mgr.span_to_md(context, &decl.span);

            if context.experimental.new_encoding {
                let mut encoded_bytes = match constant.get_content(context).value.clone() {
                    ConstantValue::RawUntypedSlice(bytes) => bytes,
                    _ => unreachable!(),
                };

                let config_type_info = engines.te().get(decl.type_ascription.type_id);
                let buffer_size = match config_type_info.abi_encode_size_hint(engines) {
                    crate::AbiEncodeSizeHint::Exact(len) => len,
                    crate::AbiEncodeSizeHint::Range(_, len) => len,
                    _ => unreachable!("unexpected type accepted as configurable"),
                };

                if buffer_size > encoded_bytes.len() {
                    encoded_bytes.extend([0].repeat(buffer_size - encoded_bytes.len()));
                }
                assert!(encoded_bytes.len() == buffer_size);

                let decode_fn = engines.de().get(decl.decode_fn.as_ref().unwrap().id());
                let keyed_decl = KeyedTyFunctionDecl::new(&decode_fn, engines);
                let decode_fn = compiled_fn_cache.get_compiled_function(
                    engines,
                    context,
                    module,
                    md_mgr,
                    &keyed_decl,
                    logged_types_map,
                    messages_types_map,
                    panic_occurrences,
                    panicking_call_occurrences,
                    panicking_fn_cache,
                )?;

                let name = decl_name.as_str().to_string();
                module.add_config(
                    context,
                    name.clone(),
                    ConfigContent::V1 {
                        name,
                        ty,
                        ptr_ty,
                        encoded_bytes,
                        decode_fn: Cell::new(decode_fn),
                        opt_metadata,
                    },
                );
            } else {
                let name = decl_name.as_str().to_string();
                module.add_config(
                    context,
                    name.clone(),
                    ConfigContent::V0 {
                        name,
                        ty,
                        ptr_ty,
                        constant,
                        opt_metadata,
                    },
                );
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_function(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &ty::TyFunctionDecl,
    original_name: &Ident,
    abi_errors_display: String,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    is_entry: bool,
    is_original_entry: bool,
    test_decl_ref: Option<DeclRefFunction>,
    compiled_fn_cache: &mut CompiledFunctionCache,
) -> Result<Option<Function>, Vec<CompileError>> {
    // Currently monomorphization of generics is inlined into main() and the functions with generic
    // args are still present in the AST declarations, but they can be ignored.
    if !ast_fn_decl.type_parameters.is_empty() {
        Ok(None)
    } else {
        compile_fn(
            engines,
            context,
            md_mgr,
            module,
            ast_fn_decl,
            original_name,
            abi_errors_display,
            is_entry,
            is_original_entry,
            None,
            logged_types_map,
            messages_types_map,
            panic_occurrences,
            panicking_call_occurrences,
            panicking_fn_cache,
            test_decl_ref,
            compiled_fn_cache,
        )
        .map(Some)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_entry_function(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &DeclId<ty::TyFunctionDecl>,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_decl_ref: Option<DeclRefFunction>,
    compiled_fn_cache: &mut CompiledFunctionCache,
) -> Result<Function, Vec<CompileError>> {
    let is_entry = true;
    // In the new encoding, the only entry function is the `__entry`,
    // which is not an original entry.
    let is_original_entry = !context.experimental.new_encoding;
    let ast_fn_decl = engines.de().get_function(ast_fn_decl);
    compile_function(
        engines,
        context,
        md_mgr,
        module,
        &ast_fn_decl,
        &ast_fn_decl.name,
        FnCompiler::fn_abi_errors_display(&ast_fn_decl, engines),
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        is_entry,
        is_original_entry,
        test_decl_ref,
        compiled_fn_cache,
    )
    .map(|f| f.expect("entry point should never contain generics"))
}

#[derive(Debug, Clone)]
pub enum CheckDecl {
    Ref {
        tid: TypeId,
        is_decode_trivial_table: HashMap<String, TyExpression>,
        type_name_span: Span,
    },
    Decl {
        tid: TypeId,
        is_decode_trivial_table: HashMap<String, TyExpression>,
    },
}

impl CheckDecl {
    pub fn is_decode_trivial_table(&self) -> &HashMap<String, TyExpression> {
        match self {
            CheckDecl::Ref {
                is_decode_trivial_table,
                ..
            }
            | CheckDecl::Decl {
                is_decode_trivial_table,
                ..
            } => is_decode_trivial_table,
        }
    }

    pub fn tid(&self) -> TypeId {
        match self {
            CheckDecl::Ref { tid, .. } | CheckDecl::Decl { tid, .. } => *tid,
        }
    }
}

pub fn run_ir_decl_checks(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    decls_check: &[CheckDecl],
    workspace_pid: Option<ProgramId>,
) -> Vec<CompileError> {
    let mut errors = vec![];

    // check types
    for check in decls_check.iter() {
        let is_decode_trivial_table = check
            .is_decode_trivial_table()
            .iter()
            .filter_map(|(key, expr)| {
                let expr = compile_constant_expression_to_constant(
                    engines, context, md_mgr, module, None, None, expr,
                )
                .ok()?
                .get_content(context)
                .as_bool()?;
                Some((key.clone(), expr))
            })
            .collect::<HashMap<String, bool>>();

        let type_info = engines.te().get(check.tid());
        let fullname = engines.help_out(type_info.as_ref()).to_string();

        if is_decode_trivial_table.get(&fullname) == Some(&true) {
            continue;
        }

        let type_decl_span = match type_info.as_ref() {
            TypeInfo::Enum(decl_id) => Some(engines.de().get(decl_id).name().span().clone()),
            TypeInfo::Struct(decl_id) => Some(engines.de().get(decl_id).name().span().clone()),
            _ => None,
        };

        let type_ref_span = match check {
            CheckDecl::Ref { type_name_span, .. } => Some(type_name_span.clone()),
            _ => None,
        };

        let mut error = TrivialCheckFailedData {
            span: type_ref_span
                .clone()
                .unwrap_or_else(|| type_decl_span.clone().unwrap()),
            can_be_made_trivial: None,
            infos: vec![],
            helps: vec![],
            never_trivial: BTreeSet::default(),
            bottom_helps: BTreeSet::default(),
        };

        if let (TypeInfo::Enum(_) | TypeInfo::Struct(_), CheckDecl::Ref { type_name_span, .. }) =
            (type_info.as_ref(), check)
        {
            error.infos.push((
                type_name_span.clone(),
                format!(
                    "This is the reason `{}` is being checked",
                    type_name_span.as_str()
                ),
            ));
        }

        push_help_if_non_trivially_decodable_type(
            engines,
            workspace_pid,
            type_ref_span,
            type_decl_span,
            type_info.as_ref(),
            &is_decode_trivial_table,
            &mut error,
        );

        let errors_count = error.infos.len()
            + error.helps.len()
            + error.bottom_helps.len()
            + error.never_trivial.len();
        if errors_count > 0 {
            errors.push(CompileError::TrivialCheckFailed(error));
        }
    }

    errors
}

#[allow(clippy::too_many_arguments)]
fn push_help_if_non_trivially_decodable_type(
    engines: &Engines,
    workspace_pid: Option<ProgramId>,
    type_ref_span: Option<Span>,
    type_decl_span: Option<Span>,
    type_info: &TypeInfo,
    table: &HashMap<String, bool>,
    error: &mut TrivialCheckFailedData,
) {
    let fullname = engines.help_out(type_info).to_string();
    if *table.get(&fullname).unwrap() {
        return;
    }

    // Check for decls, like an attribute above a struct decl
    if let Some(type_decl_span) = type_decl_span {
        match &type_info {
            TypeInfo::Struct(decl_id) => {
                let struct_decl = engines.de().get(decl_id);
                error.span = struct_decl.call_path.suffix.span().clone();

                //Only suggest change if the decl is in the same workspace
                let decl_pid = struct_decl.span.source_id().map(|x| x.program_id());
                match (workspace_pid, decl_pid) {
                    (Some(a), Some(b)) if a == b => {
                        error.helps.push((
                            type_decl_span.clone(),
                            format!(
                                "Consider changing `{}` to make it trivially decodable.",
                                type_decl_span.as_str()
                            ),
                        ));
                    }
                    _ => {}
                }

                for field in struct_decl.fields.iter() {
                    let field_tid = field.type_argument.type_id;
                    let field_type_info = engines.te().get(field_tid);
                    let field_fullname = engines.help_out(field_tid).to_string();

                    if *table.get(&field_fullname).unwrap() {
                        continue;
                    }

                    error.infos.push((
                        field.name.span().clone(),
                        "This field is not trivially decodable.".to_string(),
                    ));

                    push_help_if_non_trivially_decodable_type(
                        engines,
                        workspace_pid,
                        Some(field.type_argument.span.clone()),
                        None,
                        field_type_info.as_ref(),
                        table,
                        error,
                    );
                }
            }
            TypeInfo::Enum(decl_id) => {
                let enum_decl = engines.de().get(decl_id);
                error.span = enum_decl.call_path.suffix.span().clone();
            }
            x => {
                todo!("{:?}", engines.help_out(x));
            }
        }
    }

    // Check when a type is just a ref, like a function argument, or a field inside a decl
    if let Some(type_ref_span) = type_ref_span {
        match &type_info {
            TypeInfo::Boolean => {
                error.helps.push((
                    type_ref_span.clone(),
                    "Consider changing this type to `TrivialBool`.".to_string(),
                ));
                error.never_trivial.insert("bool".to_string());
            }
            TypeInfo::UnsignedInteger(IntegerBits::Sixteen) => {
                error.helps.push((
                    type_ref_span.clone(),
                    "Consider changing this type to `u64`.".to_string(),
                ));
                error.never_trivial.insert("u16".to_string());
            }
            TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo) => {
                error.helps.push((
                    type_ref_span.clone(),
                    "Consider changing this type to `u64`.".to_string(),
                ));
                error.never_trivial.insert("u32".to_string());
            }
            TypeInfo::Enum(_) => {
                error.helps.push((
                    type_ref_span.clone(),
                    format!(
                        "Consider wrapping this type like `TrivialEnum<{}>`.",
                        type_ref_span.as_str(),
                    ),
                ));
                error.bottom_helps.insert("Enums are represented as tagged unions and because of that they cannot be trivially decoded. But where needed, they can be wrapped by `TrivialEnum`, and their original value can be retrieved later with `unwrap`.".to_string());
            }
            TypeInfo::Struct(_) => {
                // special types
                let full_type = engines.help_out(type_info).to_string();
                if full_type.starts_with("std::vec::Vec<") {
                    let aray_type_name = full_type
                        .replace("std::vec::Vec<", "[")
                        .replace(">", "; 64]");
                    error.helps.push((
                        type_ref_span.clone(),
                        format!("`Vec` is never trivially decodable. Consider using array instead, e.g.: `{aray_type_name}`.")
                    ));
                } else if full_type.starts_with("std::string::String") {
                    error.helps.push((
                        type_ref_span.clone(),
                        "`String` is never trivially decodable. Consider using array instead, e.g.: `str[64]`.".to_string()
                    ));
                }
            }
            TypeInfo::Tuple(items) => {
                for item in items {
                    let type_info = engines.te().get(item.type_id);
                    push_help_if_non_trivially_decodable_type(
                        engines,
                        workspace_pid,
                        Some(item.span.clone()),
                        None,
                        type_info.as_ref(),
                        table,
                        error,
                    );
                }
            }
            TypeInfo::Array(item, _) => {
                let type_info = engines.te().get(item.type_id);
                push_help_if_non_trivially_decodable_type(
                    engines,
                    workspace_pid,
                    Some(item.span.clone()),
                    None,
                    type_info.as_ref(),
                    table,
                    error,
                );
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compile_tests(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_fns: &[(Arc<ty::TyFunctionDecl>, DeclRefFunction)],
    compiled_fn_cache: &mut CompiledFunctionCache,
) -> Result<Vec<Function>, Vec<CompileError>> {
    test_fns
        .iter()
        .map(|(_ast_fn_decl, decl_ref)| {
            compile_entry_function(
                engines,
                context,
                md_mgr,
                module,
                decl_ref.id(),
                logged_types_map,
                messages_types_map,
                panic_occurrences,
                panicking_call_occurrences,
                panicking_fn_cache,
                Some(decl_ref.clone()),
                compiled_fn_cache,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn compile_fn(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &ty::TyFunctionDecl,
    // Original function name, before it is postfixed with
    // a number, to get a unique name.
    // The span in the name must point to the name in the
    // function declaration.
    original_name: &Ident,
    abi_errors_display: String,
    is_entry: bool,
    is_original_entry: bool,
    selector: Option<[u8; 4]>,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    test_decl_ref: Option<DeclRefFunction>,
    compiled_fn_cache: &mut CompiledFunctionCache,
) -> Result<Function, Vec<CompileError>> {
    let inline = ast_fn_decl.inline();
    let trace = ast_fn_decl.trace();
    let ty::TyFunctionDecl {
        name,
        body,
        return_type,
        visibility,
        purity,
        span,
        is_trait_method_dummy,
        is_type_check_finalized,
        ..
    } = ast_fn_decl;

    if *is_trait_method_dummy {
        return Err(vec![CompileError::InternalOwned(
            format!("Method {name} is a trait method dummy and was not properly replaced."),
            span.clone(),
        )]);
    }

    if !*is_type_check_finalized {
        return Err(vec![CompileError::InternalOwned(
            format!("Method {name} did not finalize type checking phase."),
            span.clone(),
        )]);
    }

    let mut ref_mut_args = rustc_hash::FxHashSet::default();
    let mut args = ast_fn_decl
        .parameters
        .iter()
        .map(|param| {
            // Convert to an IR type.
            convert_resolved_type_id(
                engines,
                context,
                md_mgr,
                module,
                None,
                param.type_argument.type_id,
                &param.type_argument.span,
            )
            .map(|ty| {
                if param.is_reference && param.is_mutable {
                    ref_mut_args.insert(param.name.as_str().to_owned());
                }
                (
                    // Convert the name.
                    param.name.as_str().into(),
                    // Convert the type further to a pointer if it's a reference.
                    if param.is_reference {
                        Type::new_typed_pointer(context, ty)
                    } else {
                        ty
                    },
                    // Convert the span to a metadata index.
                    md_mgr.span_to_md(context, &param.name.span()),
                )
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()
        .map_err(|err| vec![err])?;

    let keyed_decl = KeyedTyFunctionDecl::new(ast_fn_decl, engines);
    if context.backtrace != Backtrace::None && panicking_fn_cache.can_panic(&keyed_decl, engines) {
        args.push((
            FnCompiler::BACKTRACE_FN_ARG_NAME.to_string(),
            Type::new_uint(context, 64),
            None,
        ));
    }

    let args = args; // Remove mutability.

    let ret_type = convert_resolved_type_id(
        engines,
        context,
        md_mgr,
        module,
        None,
        return_type.type_id,
        &return_type.span,
    )
    .map_err(|err| vec![err])?;

    let span_md_idx = md_mgr.span_to_md(context, span);
    let storage_md_idx = md_mgr.purity_to_md(context, *purity);
    let name_span_md_idx = md_mgr.fn_name_span_to_md(context, original_name);
    let mut metadata = md_combine(context, &span_md_idx, &storage_md_idx);
    metadata = md_combine(context, &metadata, &name_span_md_idx);

    let decl_index = test_decl_ref.map(|decl_ref| *decl_ref.id());
    if let Some(decl_index) = decl_index {
        let test_decl_index_md_idx = md_mgr.test_decl_index_to_md(context, decl_index);
        metadata = md_combine(context, &metadata, &test_decl_index_md_idx);
    }
    if let Some(inline) = inline {
        let inline_md_idx = md_mgr.inline_to_md(context, inline);
        metadata = md_combine(context, &metadata, &inline_md_idx);
    }
    if let Some(trace) = trace {
        let trace_md_idx = md_mgr.trace_to_md(context, trace);
        metadata = md_combine(context, &metadata, &trace_md_idx);
    }

    let func = Function::new(
        context,
        module,
        name.as_str().to_owned(),
        abi_errors_display,
        args,
        ret_type,
        selector,
        *visibility == Visibility::Public,
        is_entry,
        is_original_entry,
        ast_fn_decl.is_fallback(),
        metadata,
    );

    let mut compiler = FnCompiler::new(
        engines,
        context,
        module,
        func,
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        compiled_fn_cache,
    );
    compiler.ref_mut_args = ref_mut_args;

    let mut ret_val = compiler.compile_fn_to_value(context, md_mgr, body)?;

    // Special case: sometimes the returned value at the end of the function block is hacked
    // together and is invalid.  This can happen with diverging control flow or with implicit
    // returns.  We can double check here and make sure the return value type is correct.
    let undef = Constant::unique(context, ConstantContent::get_undef(ret_type));
    ret_val = match ret_val.get_type(context) {
        Some(ret_val_type)
            if ret_type.eq(context, &ret_val_type)
            // TODO: This must be removed along with sway_core::ir_generation::type_correction.
                || ret_val_type
                    .get_pointee_type(context)
                    .is_some_and(|pointee_ty| pointee_ty.eq(context, &ret_type)) =>
        {
            ret_val
        }

        // Mismatched or unavailable type.  Set ret_val to a correctly typed Undef.
        _otherwise => Value::new_constant(context, undef),
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
        .is_terminated_by_return_or_revert(context);
    if !already_returns
        && (compiler.current_block.num_instructions(context) > 0
            || compiler.current_block == compiler.function.get_entry_block(context)
            || compiler.current_block.num_predecessors(context) > 0)
    {
        if ret_type.is_unit(context) {
            ret_val = ConstantContent::get_unit(context);
        }
        compiler
            .current_block
            .append(context)
            .ret(ret_val, ret_type);
    }
    Ok(func)
}

#[allow(clippy::too_many_arguments)]
fn compile_encoding_v0_abi_method(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    ast_fn_decl: &DeclId<ty::TyFunctionDecl>,
    logged_types_map: &HashMap<TypeId, LogId>,
    messages_types_map: &HashMap<TypeId, MessageId>,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    panicking_fn_cache: &mut PanickingFunctionCache,
    engines: &Engines,
    compiled_fn_cache: &mut CompiledFunctionCache,
) -> Result<Function, Vec<CompileError>> {
    assert!(
        !context.experimental.new_encoding,
        "`new_encoding` was true while calling `compile_encoding_v0_abi_method`"
    );

    // Use the error from .to_fn_selector_value() if possible, else make an CompileError::Internal.
    let handler = Handler::default();
    let ast_fn_decl = engines.de().get_function(ast_fn_decl);

    let get_selector_result = ast_fn_decl.to_fn_selector_value(&handler, engines);
    let (errors, _warnings, _infos) = handler.consume();
    let selector = match get_selector_result.ok() {
        Some(selector) => selector,
        None => {
            return if !errors.is_empty() {
                Err(vec![errors[0].clone()])
            } else {
                Err(vec![CompileError::InternalOwned(
                    format!(
                        "Cannot generate selector for ABI method: {}",
                        ast_fn_decl.name.as_str()
                    ),
                    ast_fn_decl.name.span(),
                )])
            };
        }
    };

    compile_fn(
        engines,
        context,
        md_mgr,
        module,
        &ast_fn_decl,
        &ast_fn_decl.name,
        FnCompiler::fn_abi_errors_display(&ast_fn_decl, engines),
        // ABI methods are only entries when the "new encoding" is off
        !context.experimental.new_encoding,
        // ABI methods are always original entries
        true,
        Some(selector),
        logged_types_map,
        messages_types_map,
        panic_occurrences,
        panicking_call_occurrences,
        panicking_fn_cache,
        None,
        compiled_fn_cache,
    )
}
