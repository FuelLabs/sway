use super::ast_node::TypedTraitDeclaration;
use crate::error::*;
use crate::{CompileResult, TypeInfo};
use crate::{Ident, TypedDeclaration, TypedFunctionDeclaration};
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct Namespace<'sc> {
    symbols: HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    implemented_traits: HashMap<(Ident<'sc>, TypeInfo<'sc>), Vec<TypedFunctionDeclaration<'sc>>>,
    /// any imported namespaces associated with an ident which is a  library name
    modules: HashMap<Ident<'sc>, Namespace<'sc>>,
}

impl<'sc> Namespace<'sc> {
    pub(crate) fn star_import(&mut self, idents: Vec<&Ident<'sc>>) -> CompileResult<()> {
        let idents_buf = idents.into_iter();
        let mut namespace = self.clone();
        for ident in idents_buf {
            let other_namespace = match namespace.modules.get(ident) {
                Some(o) => namespace = o.clone(),
                None => todo!("library not found"),
            };
        }
        self.merge_namespaces(&namespace);
        ok((), vec![], vec![])
    }

    pub(crate) fn item_import(
        &mut self,
        ident: Vec<&Ident<'sc>>,
        alias: Option<Ident<'sc>>,
    ) -> CompileResult<()> {
        let mut idents = ident.clone();
        let mut warnings = vec![];
        let last_item = idents.pop().unwrap();
        let mut namespace = self.clone();
        for ident in idents {
            let other_namespace = match namespace.modules.get(ident) {
                Some(o) => namespace = o.clone(),
                None => todo!("library not found"),
            };
        }

        match namespace.symbols.get(last_item) {
            Some(o) => {
                let name = match alias {
                    Some(s) => s.clone(),
                    None => last_item.clone(),
                };

                if let Some(_) = self.symbols.get(&name) {
                    warnings.push(CompileWarning {
                        span: name.span.clone(),
                        warning_content: Warning::OverridesOtherSymbol {
                            name: name.span.clone().as_str(),
                        },
                    });
                }
                self.symbols.insert(name, o.clone());
            }
            None => todo!("item not found"),
        };

        ok((), warnings, vec![])
    }

    fn merge_namespaces(&mut self, other: &Namespace<'sc>) {
        for (name, symbol) in &other.symbols {
            self.symbols.insert(name.clone(), symbol.clone());
        }
        for ((name, typ), trait_impl) in &other.implemented_traits {
            self.implemented_traits
                .insert((name.clone(), typ.clone()), trait_impl.clone());
        }
    }
}
