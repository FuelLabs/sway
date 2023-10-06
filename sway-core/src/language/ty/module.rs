use sway_error::handler::Handler;
use sway_types::Span;

use crate::{
    decl_engine::{DeclEngine, DeclRef, DeclRefFunction},
    language::ty::*,
    language::ModName,
    semantic_analysis::namespace,
    transform::{self, AllowDeprecatedState},
    Engines,
};

#[derive(Clone, Debug)]
pub struct TyModule {
    pub span: Span,
    pub submodules: Vec<(ModName, TySubmodule)>,
    pub namespace: namespace::Namespace,
    pub all_nodes: Vec<TyAstNode>,
    pub attributes: transform::AttributesMap,
}

#[derive(Clone, Debug)]
pub struct TySubmodule {
    pub module: TyModule,
    pub mod_name_span: Span,
}

/// Iterator type for iterating over submodules.
///
/// Used rather than `impl Iterator` to enable recursive submodule iteration.
pub struct SubmodulesRecursive<'module> {
    submods: std::slice::Iter<'module, (ModName, TySubmodule)>,
    current: Option<(
        &'module (ModName, TySubmodule),
        Box<SubmodulesRecursive<'module>>,
    )>,
}

impl TyModule {
    /// An iterator yielding all submodules recursively, depth-first.
    pub fn submodules_recursive(&self) -> SubmodulesRecursive {
        SubmodulesRecursive {
            submods: self.submodules.iter(),
            current: None,
        }
    }

    /// All test functions within this module.
    pub fn test_fns<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl '_ + Iterator<Item = (TyFunctionDecl, DeclRefFunction)> {
        self.all_nodes.iter().filter_map(|node| {
            if let TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl {
                decl_id,
                subst_list: _,
                name,
                decl_span,
            })) = &node.content
            {
                let fn_decl = decl_engine.get_function(decl_id);
                if fn_decl.is_test() {
                    return Some((
                        fn_decl,
                        DeclRef::new(name.clone(), *decl_id, decl_span.clone()),
                    ));
                }
            }
            None
        })
    }

    pub(crate) fn check_deprecated(
        &self,
        engines: &Engines,
        handler: &Handler,
        allow_deprecated: &mut AllowDeprecatedState,
    ) {
        for (_, submodule) in self.submodules.iter() {
            submodule
                .module
                .check_deprecated(engines, handler, allow_deprecated);
        }

        for node in self.all_nodes.iter() {
            node.check_deprecated(engines, handler, allow_deprecated);
        }
    }
}

impl<'module> Iterator for SubmodulesRecursive<'module> {
    type Item = &'module (ModName, TySubmodule);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current = match self.current.take() {
                None => match self.submods.next() {
                    None => return None,
                    Some(submod) => {
                        Some((submod, Box::new(submod.1.module.submodules_recursive())))
                    }
                },
                Some((submod, mut submods)) => match submods.next() {
                    Some(next) => {
                        self.current = Some((submod, submods));
                        return Some(next);
                    }
                    None => return Some(submod),
                },
            }
        }
    }
}
