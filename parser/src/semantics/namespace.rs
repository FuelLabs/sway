use super::ast_node::TypedTraitDeclaration;
use crate::error::*;
use crate::CallPath;
use crate::{CompileResult, TypeInfo};
use crate::{Ident, TypedDeclaration, TypedFunctionDeclaration};
use std::collections::HashMap;

type ModuleName = String;

#[derive(Clone, Debug, Default)]
pub struct Namespace<'sc> {
    symbols: HashMap<Ident<'sc>, TypedDeclaration<'sc>>,
    implemented_traits: HashMap<(Ident<'sc>, TypeInfo<'sc>), Vec<TypedFunctionDeclaration<'sc>>>,
    /// any imported namespaces associated with an ident which is a  library name
    modules: HashMap<ModuleName, Namespace<'sc>>,
}

impl<'sc> Namespace<'sc> {
    pub(crate) fn star_import(&mut self, idents: Vec<Ident<'sc>>) -> CompileResult<()> {
        let debug_idents = idents.clone();
        let idents_buf = idents.into_iter();
        let mut namespace = self.clone();
        dbg!(&namespace);
        for ident in idents_buf {
            let other_namespace = match namespace.modules.get(ident.primary_name) {
                Some(o) => namespace = o.clone(),
                None => todo!("library not found: {:?}", debug_idents),
            };
        }
        self.merge_namespaces(&namespace);
        ok((), vec![], vec![])
    }

    pub(crate) fn item_import(
        &mut self,
        path: Vec<Ident<'sc>>,
        item: &Ident<'sc>,
        alias: Option<Ident<'sc>>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let namespace = match self.find_module(&path) {
            Some(o) => o,
            None => todo!("module not found error"),
        };

        match namespace.symbols.get(item) {
            Some(o) => {
                let name = match alias {
                    Some(s) => s.clone(),
                    None => item.clone(),
                };
                self.insert(name, o.clone());
            }
            None => todo!("item not found"),
        };

        ok((), warnings, vec![])
    }

    pub(crate) fn merge_namespaces(&mut self, other: &Namespace<'sc>) {
        for (name, symbol) in &other.symbols {
            self.symbols.insert(name.clone(), symbol.clone());
        }
        for ((name, typ), trait_impl) in &other.implemented_traits {
            self.implemented_traits
                .insert((name.clone(), typ.clone()), trait_impl.clone());
        }
    }

    pub(crate) fn insert(
        &mut self,
        name: Ident<'sc>,
        item: TypedDeclaration<'sc>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        if let Some(_) = self.symbols.get(&name) {
            warnings.push(CompileWarning {
                span: name.span.clone(),
                warning_content: Warning::OverridesOtherSymbol {
                    name: name.span.clone().as_str(),
                },
            });
        }
        self.symbols.insert(name, item.clone());
        ok((), warnings, vec![])
    }

    pub(crate) fn get_symbol(&self, symbol: &Ident<'sc>) -> Option<&TypedDeclaration<'sc>> {
        self.symbols.get(symbol)
    }

    pub(crate) fn get_call_path(&self, path: &CallPath<'sc>) -> Option<TypedDeclaration<'sc>> {
        let module = match self.find_module(&path.prefixes) {
            Some(o) => o,
            None => todo!("err module not found"),
        };

        module.symbols.get(&path.suffix).cloned()
    }

    fn find_module(&self, path: &Vec<Ident<'sc>>) -> Option<Namespace<'sc>> {
        let mut namespace = self.clone();
        for ident in path {
            let other_namespace = match namespace.modules.get(ident.primary_name) {
                Some(o) => namespace = o.clone(),
                None => todo!("library not found"),
            };
        }
        Some(namespace)
    }
    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: Ident<'sc>,
        type_implementing_for: TypeInfo<'sc>,
        functions_buf: Vec<TypedFunctionDeclaration<'sc>>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        if let Some(_) = self
            .implemented_traits
            .get(&(trait_name.clone(), type_implementing_for.clone()))
        {
            warnings.push(CompileWarning {
                warning_content: Warning::OverridingTraitImplementation,
                span: functions_buf.iter().fold(
                    functions_buf[0].span.clone(),
                    |acc, TypedFunctionDeclaration { span, .. }| {
                        crate::utils::join_spans(acc, span.clone())
                    },
                ),
            })
        }
        self.implemented_traits
            .insert((trait_name, type_implementing_for), functions_buf);
        ok((), warnings, vec![])
    }

    pub fn insert_module(&mut self, module_name: String, module_contents: Namespace<'sc>) {
        self.modules.insert(module_name, module_contents);
    }
}
