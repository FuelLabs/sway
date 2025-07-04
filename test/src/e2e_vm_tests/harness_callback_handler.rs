use normalize_path::NormalizePath as _;
use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr as _,
    sync::{Arc, Mutex},
};
use sway_core::engine_threading::CallbackHandler;

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

enum Cmds {
    PrintArgs,
    Trace(bool),
}

struct Inner {
    eng: rhai::Engine,
    ast: rhai::AST,
    pkg_name_cache: HashMap<PathBuf, String>,
    cmds: Arc<Mutex<Vec<Cmds>>>,
    snapshot: String,
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

    fn run_cmds(&mut self, ctx: &sway_core::semantic_analysis::TypeCheckContext<'_>, args: String) {
        let cmds = self.cmds.lock().unwrap();
        for cmd in cmds.iter() {
            match cmd {
                Cmds::PrintArgs => {
                    self.snapshot.push_str(&format!("{}\n", args));
                }
                Cmds::Trace(enable) => {
                    ctx.engines.obs().enable_trace(*enable);
                }
            }
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

        self.cmds.lock().unwrap().clear();
        let _ = self.eng.eval_ast_with_scope::<()>(&mut scope, &self.ast);

        let args = format!(
            "on_before_method_resolution: {:?}; {:?}; {:?}",
            method_name.inner,
            method_name.type_arguments,
            ctx.engines.help_out(args_types.to_vec())
        );

        self.run_cmds(ctx, args);
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

        self.cmds.lock().unwrap().clear();
        let _ = self.eng.eval_ast_with_scope::<()>(&mut scope, &self.ast);

        let args = format!(
            "on_after_method_resolution: {:?}; {:?}; {:?}; {:?}; {:?}",
            method_name.inner,
            method_name.type_arguments,
            ctx.engines.help_out(args_types.to_vec()),
            ctx.engines.help_out(new_ref.id()),
            ctx.engines.help_out(new_type_id),
        );

        self.run_cmds(ctx, args);
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if !self.snapshot.is_empty() {
            stdout_logs(&self.root, &self.snapshot);
        }
    }
}

pub struct HarnessCallbackHandler {
    inner: Mutex<Inner>,
}

impl HarnessCallbackHandler {
    pub fn new(root: &str, script: &str) -> Self {
        let cmds = Arc::new(Mutex::new(vec![]));

        let mut eng = rhai::Engine::new();
        eng.register_fn("print_args", {
            let cmds = cmds.clone();
            move || {
                cmds.lock().unwrap().push(Cmds::PrintArgs);
            }
        });
        eng.register_fn("trace", {
            let cmds = cmds.clone();
            move |b| {
                cmds.lock().unwrap().push(Cmds::Trace(b));
            }
        });

        let scope = rhai::Scope::new();
        let ast = eng.compile_into_self_contained(&scope, script).unwrap();

        Self {
            inner: Mutex::new(Inner {
                eng,
                ast,
                pkg_name_cache: HashMap::default(),
                cmds,
                snapshot: String::new(),
                root: root.to_string(),
            }),
        }
    }
}

impl CallbackHandler for HarnessCallbackHandler {
    fn on_trace(&self, msg: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.snapshot.push_str(msg);
        inner.snapshot.push('\n');
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
}
