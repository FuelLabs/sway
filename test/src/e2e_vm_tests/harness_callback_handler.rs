use normalize_path::NormalizePath as _;
use sway_ir::Type;
use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr as _,
    sync::{Arc, Mutex},
};
use sway_core::{Engines, Observer, TypeInfo, ir_generation::{get_encoding_representation, get_runtime_representation}};

fn stdout_logs(root: &str, snapshot: &str) {
    let root = PathBuf::from_str(root).unwrap();
    let root = root.normalize();

    let mut insta = insta::Settings::new();
    insta.set_snapshot_path(root);
    insta.set_prepend_module_to_snapshot(false);
    insta.set_omit_expression(true);
    let scope = insta.bind_to_scope();
    let _ = std::panic::catch_unwind(|| {
        insta::assert_snapshot!("logs", snapshot);
    });
    drop(scope);
}

struct Inner {
    eng: rhai::Engine,
    ast: rhai::AST,
    pkg_name_cache: HashMap<PathBuf, String>,
    snapshot: Arc<Mutex<String>>,
    root: String,
}

impl Inner {
    fn get_package_name(
        &mut self,
        span: &sway_types::Span,
        engines: &sway_core::Engines,
    ) -> Option<String> {
        if let Some(sid) = span.source_id() {
            let filename = engines.se().get_path(sid);
            if let Some(pid) = engines.se().get_program_id_from_manifest_path(&filename) {
                let path = engines
                    .se()
                    .get_manifest_path_from_program_id(&pid)
                    .unwrap()
                    .join("Forc.toml");

                Some(
                    if let Some(pkg_name) = self.pkg_name_cache.get(&path).cloned() {
                        pkg_name
                    } else {
                        let toml = std::fs::read_to_string(&path).unwrap();
                        let forc_toml: toml::Table = toml::from_str(&toml).unwrap();
                        let pkg_name = forc_toml["project"]["name"].as_str().unwrap().to_string();
                        self.pkg_name_cache.insert(path.clone(), pkg_name.clone());
                        pkg_name
                    },
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    fn on_before_method_resolution(
        &mut self,
        ctx: &sway_core::semantic_analysis::TypeCheckContext<'_>,
        method_name: &sway_core::type_system::ast_elements::binding::TypeBinding<
            sway_core::language::parsed::MethodName,
        >,
        args_types: &[sway_core::TypeId],
    ) {
        let pkg_name = self
            .get_package_name(&method_name.span, ctx.engines)
            .unwrap_or_default();
        if pkg_name.is_empty() || pkg_name == "std" {
            return;
        }

        let mut scope = rhai::Scope::new();
        scope
            .push_constant("pkg", pkg_name.clone())
            .push_constant("event", "on_before_method_resolution")
            .push_constant("method", method_name.inner.easy_name().as_str().to_string());

        let _ = self.eng.eval_ast_with_scope::<()>(&mut scope, &self.ast);

        // let args = format!(
        //     "on_before_method_resolution: {:?}; {:?}; {:?}",
        //     method_name.inner,
        //     method_name.type_arguments,
        //     ctx.engines.help_out(args_types.to_vec())
        // );

        // self.run_cmds(ctx.engines, args);
    }

    fn on_after_method_resolution(
        &mut self,
        ctx: &sway_core::semantic_analysis::TypeCheckContext<'_>,
        method_name: &sway_core::type_system::ast_elements::binding::TypeBinding<
            sway_core::language::parsed::MethodName,
        >,
        args_types: &[sway_core::TypeId],
        new_ref: sway_core::decl_engine::DeclRefFunction,
        new_type_id: sway_core::TypeId,
    ) {
        let pkg_name = self
            .get_package_name(&method_name.span, ctx.engines)
            .unwrap_or_default();
        if pkg_name.is_empty() || pkg_name == "std" {
            return;
        }

        let mut scope = rhai::Scope::new();
        scope
            .push_constant("pkg", pkg_name.clone())
            .push_constant("event", "on_after_method_resolution")
            .push_constant("method", method_name.inner.easy_name().as_str().to_string());

        let _ = self.eng.eval_ast_with_scope::<()>(&mut scope, &self.ast);

        // let args = format!(
        //     "on_after_method_resolution: {:?}; {:?}; {:?}; {:?}; {:?}",
        //     method_name.inner,
        //     method_name.type_arguments,
        //     ctx.engines.help_out(args_types.to_vec()),
        //     ctx.engines.help_out(new_ref.id()),
        //     ctx.engines.help_out(new_type_id),
        // );

        // self.run_cmds(ctx.engines, args);
    }

    fn on_after_ir_type_resolution(&mut self, engines: &Engines, ctx: &sway_ir::Context, type_info: &TypeInfo, ir_type: &Type) {
        let mut scope = rhai::Scope::new();

        let runtime_mem_repr = get_runtime_representation(ctx, *ir_type);
        let encoding_mem_repr = get_encoding_representation(engines, type_info);
        let is_trivial = if let Some(encoding_mem_repr) = encoding_mem_repr.as_ref() {
            runtime_mem_repr == *encoding_mem_repr
        } else {
            false
        };

        let type_size = ir_type.size(ctx);

        scope
            .push_constant("event", "on_after_ir_type_resolution")
            .push_constant("type_info", engines.help_out(type_info).to_string())
            .push_constant("ir_type", ir_type.as_string(ctx))
            .push_constant("runtime_mem_repr", format!("{runtime_mem_repr:?}"))
            .push_constant("encoding_mem_repr", format!("{encoding_mem_repr:?}"))
            .push_constant("is_trivial", is_trivial)
            .push_constant("type_size", type_size.in_bytes());

        let _ = self.eng.eval_ast_with_scope::<()>(&mut scope, &self.ast);

        // let args = format!(
        //     "on_after_ir_type_resolution",
        // );

        // self.run_cmds(engines, args);
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let snapshot = self.snapshot.lock().unwrap();
        if !snapshot.is_empty() {
            stdout_logs(&self.root, &snapshot);
        }
    }
}

pub struct HarnessCallbackHandler {
    inner: Mutex<Inner>,
}

impl HarnessCallbackHandler {
    pub fn new(root: &str, script: &str) -> Self {
        let snapshot = Arc::new(Mutex::new(String::new()));

        let mut eng = rhai::Engine::new();
        eng.on_print({
            let snapshot = snapshot.clone();
            move |s| {
                let mut snapshot = snapshot.lock().unwrap();
                snapshot.push_str(s);
            }
        });
        eng.register_fn("unique_print", {
            let snapshot = snapshot.clone();
            move |s: &str| {
                let mut snapshot = snapshot.lock().unwrap();
                if !snapshot.contains(s) {
                    snapshot.push_str(s);
                }
            }
        });
        eng.register_fn("println", {
            let snapshot = snapshot.clone();
            move |s: &str| {
                let mut snapshot = snapshot.lock().unwrap();
                snapshot.push_str(s);
                snapshot.push('\n');
            }
        });
        eng.register_fn("trace", |enabled: bool| {
            todo!()
        });

        let scope = rhai::Scope::new();
        let ast = eng.compile_into_self_contained(&scope, script).unwrap();

        Self {
            inner: Mutex::new(Inner {
                eng,
                ast,
                pkg_name_cache: HashMap::default(),
                snapshot,
                root: root.to_string(),
            }),
        }
    }
}

impl Observer for HarnessCallbackHandler {
    fn on_trace(&self, msg: &str) {
        let inner = self.inner.lock().unwrap();
        let mut snapshot = inner.snapshot.lock().unwrap();
        snapshot.push_str(msg);
        snapshot.push('\n');
    }

    fn on_before_method_resolution(
        &self,
        ctx: &sway_core::semantic_analysis::TypeCheckContext<'_>,
        method_name: &sway_core::type_system::ast_elements::binding::TypeBinding<
            sway_core::language::parsed::MethodName,
        >,
        args_types: &[sway_core::TypeId],
    ) {
        let mut inner = self.inner.lock().unwrap();
        inner.on_before_method_resolution(ctx, method_name, args_types);
    }

    fn on_after_method_resolution(
        &self,
        ctx: &sway_core::semantic_analysis::TypeCheckContext<'_>,
        method_name: &sway_core::type_system::ast_elements::binding::TypeBinding<
            sway_core::language::parsed::MethodName,
        >,
        args_types: &[sway_core::TypeId],
        new_ref: sway_core::decl_engine::DeclRefFunction,
        new_type_id: sway_core::TypeId,
    ) {
        let mut inner = self.inner.lock().unwrap();
        inner.on_after_method_resolution(ctx, method_name, args_types, new_ref, new_type_id);
    }

    fn on_after_ir_type_resolution(
        &self,
        engines: &Engines,
        ctx: &sway_ir::Context,
        type_info: &TypeInfo,
        ir_type: &Type,
    ) {
        let mut inner = self.inner.lock().unwrap();
        inner.on_after_ir_type_resolution(engines, ctx, type_info, ir_type);
    }
}
