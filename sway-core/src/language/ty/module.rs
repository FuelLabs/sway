use std::sync::Arc;

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    decl_engine::{DeclEngine, DeclEngineGet, DeclId, DeclRef, DeclRefFunction},
    language::{ty::*, HasModule, HasSubmodules, ModName},
    transform::{self, AllowDeprecatedState},
    Engines,
};

#[derive(Clone, Debug)]
pub struct TyModule {
    pub span: Span,
    pub submodules: Vec<(ModName, TySubmodule)>,
    pub all_nodes: Vec<TyAstNode>,
    pub attributes: transform::Attributes,
}

impl TyModule {
    /// Iter on all constants in this module, which means, globals constants and
    /// local constants, but it does not enter into submodules.
    pub fn iter_constants(&self, de: &DeclEngine) -> Vec<ConstantDecl> {
        fn inside_code_block(de: &DeclEngine, block: &TyCodeBlock) -> Vec<ConstantDecl> {
            block
                .contents
                .iter()
                .flat_map(|node| inside_ast_node(de, node))
                .collect::<Vec<_>>()
        }

        fn inside_ast_node(de: &DeclEngine, node: &TyAstNode) -> Vec<ConstantDecl> {
            match &node.content {
                TyAstNodeContent::Declaration(decl) => match decl {
                    TyDecl::ConstantDecl(decl) => {
                        vec![decl.clone()]
                    }
                    TyDecl::FunctionDecl(decl) => {
                        let decl = de.get(&decl.decl_id);
                        inside_code_block(de, &decl.body)
                    }
                    TyDecl::ImplSelfOrTrait(decl) => {
                        let decl = de.get(&decl.decl_id);
                        decl.items
                            .iter()
                            .flat_map(|item| match item {
                                TyTraitItem::Fn(decl) => {
                                    let decl = de.get(decl.id());
                                    inside_code_block(de, &decl.body)
                                }
                                TyTraitItem::Constant(decl) => {
                                    vec![ConstantDecl {
                                        decl_id: *decl.id(),
                                    }]
                                }
                                _ => vec![],
                            })
                            .collect()
                    }
                    _ => vec![],
                },
                _ => vec![],
            }
        }

        self.all_nodes
            .iter()
            .flat_map(|node| inside_ast_node(de, node))
            .collect::<Vec<_>>()
    }

    /// Recursively find all test function declarations.
    pub fn test_fns_recursive<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl 'b + Iterator<Item = (Arc<TyFunctionDecl>, DeclRefFunction)> {
        self.submodules_recursive()
            .flat_map(|(_, submod)| submod.module.test_fns(decl_engine))
            .chain(self.test_fns(decl_engine))
    }
}

#[derive(Clone, Debug)]
pub struct TySubmodule {
    pub module: Arc<TyModule>,
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
    pub fn submodules_recursive(&self) -> SubmodulesRecursive<'_> {
        SubmodulesRecursive {
            submods: self.submodules.iter(),
            current: None,
        }
    }

    /// All test functions within this module.
    pub fn test_fns<'a: 'b, 'b>(
        &'b self,
        decl_engine: &'a DeclEngine,
    ) -> impl 'b + Iterator<Item = (Arc<TyFunctionDecl>, DeclRefFunction)> {
        self.all_nodes.iter().filter_map(|node| {
            if let TyAstNodeContent::Declaration(TyDecl::FunctionDecl(FunctionDecl { decl_id })) =
                &node.content
            {
                let fn_decl = decl_engine.get_function(decl_id);
                let name = fn_decl.name.clone();
                let span = fn_decl.span.clone();
                if fn_decl.is_test() {
                    return Some((fn_decl, DeclRef::new(name, *decl_id, span)));
                }
            }
            None
        })
    }

    /// All contract functions within this module.
    pub fn contract_fns<'a: 'b, 'b>(
        &'b self,
        engines: &'a Engines,
    ) -> impl 'b + Iterator<Item = DeclId<TyFunctionDecl>> {
        self.all_nodes
            .iter()
            .flat_map(move |node| node.contract_fns(engines))
    }

    /// All contract supertrait functions within this module.
    pub fn contract_supertrait_fns<'a: 'b, 'b>(
        &'b self,
        engines: &'a Engines,
    ) -> impl 'b + Iterator<Item = DeclId<TyFunctionDecl>> {
        self.all_nodes
            .iter()
            .flat_map(move |node| node.contract_supertrait_fns(engines))
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

    pub(crate) fn check_recursive(
        &self,
        engines: &Engines,
        handler: &Handler,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for (_, submodule) in self.submodules.iter() {
                let _ = submodule.module.check_recursive(engines, handler);
            }

            for node in self.all_nodes.iter() {
                let _ = node.check_recursive(engines, handler);
            }

            Ok(())
        })
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

impl HasModule<TyModule> for TySubmodule {
    fn module(&self) -> &TyModule {
        &self.module
    }
}

impl HasSubmodules<TySubmodule> for TyModule {
    fn submodules(&self) -> &[(ModName, TySubmodule)] {
        &self.submodules
    }
}
