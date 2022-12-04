use sway_types::Ident;

use crate::{
    declaration_engine::{de_get_function, DeclarationId},
    language::ty::*,
    language::DepName,
    semantic_analysis::namespace,
};

#[derive(Clone, Debug)]
pub struct TyModule {
    pub submodules: Vec<(DepName, TySubmodule)>,
    pub namespace: namespace::Module,
    pub all_nodes: Vec<TyAstNode>,
}

#[derive(Clone, Debug)]
pub struct TySubmodule {
    pub library_name: Ident,
    pub module: TyModule,
}

/// Iterator type for iterating over submodules.
///
/// Used rather than `impl Iterator` to enable recursive submodule iteration.
pub struct SubmodulesRecursive<'module> {
    submods: std::slice::Iter<'module, (DepName, TySubmodule)>,
    current: Option<(
        &'module (DepName, TySubmodule),
        Box<SubmodulesRecursive<'module>>,
    )>,
}

type FunctionDeclarationWithId = (TyFunctionDeclaration, DeclarationId);
impl TyModule {
    /// An iterator yielding all submodules recursively, depth-first.
    pub fn submodules_recursive(&self) -> SubmodulesRecursive {
        SubmodulesRecursive {
            submods: self.submodules.iter(),
            current: None,
        }
    }

    /// All test functions within this module.
    pub fn test_fns(&self) -> impl '_ + Iterator<Item = FunctionDeclarationWithId> {
        self.all_nodes.iter().filter_map(|node| {
            if let TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(ref decl_id)) =
                node.content
            {
                let fn_decl = de_get_function(decl_id.clone(), &node.span)
                    .expect("no function declaration for ID");
                if fn_decl.is_test() {
                    return Some((fn_decl, decl_id.clone()));
                }
            }
            None
        })
    }
}

impl<'module> Iterator for SubmodulesRecursive<'module> {
    type Item = &'module (DepName, TySubmodule);
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
