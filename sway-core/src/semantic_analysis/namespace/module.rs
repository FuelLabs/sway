use crate::{
    engine_threading::Engines,
    language::{parsed::*, ty, Visibility},
    semantic_analysis::*,
    transform::to_parsed_lang,
    Ident, Namespace,
};

use super::{
    lexical_scope::{Items, LexicalScope, SymbolMap},
    root::Root,
    LexicalScopeId, ModuleName, ModulePath, ModulePathBuf,
};

use sway_ast::ItemConst;
use sway_error::handler::Handler;
use sway_error::{error::CompileError, handler::ErrorEmitted};
use sway_parse::{lex, Parser};
use sway_types::{span::Span, Spanned};

/// A single `Module` within a Sway project.
///
/// A `Module` is most commonly associated with an individual file of Sway code, e.g. a top-level
/// script/predicate/contract file or some library dependency whether introduced via `mod` or the
/// `[dependencies]` table of a `forc` manifest.
///
/// A `Module` contains a set of all items that exist within the lexical scope via declaration or
/// importing, along with a map of each of its submodules.
#[derive(Clone, Debug)]
pub struct Module {
    /// Submodules of the current module represented as an ordered map from each submodule's name
    /// to the associated `Module`.
    ///
    /// Submodules are normally introduced in Sway code with the `mod foo;` syntax where `foo` is
    /// some library dependency that we include as a submodule.
    ///
    /// Note that we *require* this map to be ordered to produce deterministic codegen results.
    pub(crate) submodules: im::OrdMap<ModuleName, Module>,
    /// Keeps all lexical scopes associated with this module.
    pub lexical_scopes: Vec<LexicalScope>,
    /// Current lexical scope id in the lexical scope hierarchy stack.
    pub current_lexical_scope_id: LexicalScopeId,
    /// Name of the module, package name for root module, module name for other modules.
    /// Module name used is the same as declared in `mod name;`.
    pub name: Option<Ident>,
    /// Whether or not this is a `pub` module
    pub visibility: Visibility,
    /// Empty span at the beginning of the file implementing the module
    pub span: Option<Span>,
    /// Indicates whether the module is external to the current package. External modules are
    /// imported in the `Forc.toml` file.
    pub is_external: bool,
    /// An absolute path from the `root` that represents the module location.
    ///
    /// When this is the root module, this is equal to `[]`. When this is a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    pub(crate) mod_path: ModulePathBuf,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            visibility: Visibility::Private,
            submodules: Default::default(),
            lexical_scopes: vec![LexicalScope::default()],
            current_lexical_scope_id: 0,
            name: Default::default(),
            span: Default::default(),
            is_external: Default::default(),
            mod_path: Default::default(),
        }
    }
}

impl Module {
    pub fn read<R>(&self, _engines: &crate::Engines, mut f: impl FnMut(&Module) -> R) -> R {
        f(self)
    }

    pub fn write<R>(
        &mut self,
        _engines: &crate::Engines,
        mut f: impl FnMut(&mut Module) -> R,
    ) -> R {
        f(self)
    }

    pub fn mod_path(&self) -> &ModulePath {
        self.mod_path.as_slice()
    }

    pub fn mod_path_buf(&self) -> ModulePathBuf {
        self.mod_path.clone()
    }

    /// `contract_id_value` is injected here via forc-pkg when producing the `dependency_namespace` for a contract which has tests enabled.
    /// This allows us to provide a contract's `CONTRACT_ID` constant to its own unit tests.
    ///
    /// This will eventually be refactored out of `sway-core` in favor of creating temporary package dependencies for providing these
    /// `CONTRACT_ID`-containing modules: https://github.com/FuelLabs/sway/issues/3077
    pub fn default_with_contract_id(
        engines: &Engines,
        name: Option<Ident>,
        contract_id_value: String,
        experimental: crate::ExperimentalFlags,
    ) -> Result<Self, vec1::Vec1<CompileError>> {
        let handler = <_>::default();
        Module::default_with_contract_id_inner(
            &handler,
            engines,
            name,
            contract_id_value,
            experimental,
        )
        .map_err(|_| {
            let (errors, warnings) = handler.consume();
            assert!(warnings.is_empty());

            // Invariant: `.value == None` => `!errors.is_empty()`.
            vec1::Vec1::try_from_vec(errors).unwrap()
        })
    }

    fn default_with_contract_id_inner(
        handler: &Handler,
        engines: &Engines,
        ns_name: Option<Ident>,
        contract_id_value: String,
        experimental: crate::ExperimentalFlags,
    ) -> Result<Self, ErrorEmitted> {
        // it would be nice to one day maintain a span from the manifest file, but
        // we don't keep that around so we just use the span from the generated const decl instead.
        let mut compiled_constants: SymbolMap = Default::default();
        // this for loop performs a miniature compilation of each const item in the config
        // FIXME(Centril): Stop parsing. Construct AST directly instead!
        // parser config
        let const_item = format!("pub const CONTRACT_ID: b256 = {contract_id_value};");
        let const_item_len = const_item.len();
        let input_arc = std::sync::Arc::from(const_item);
        let token_stream = lex(handler, &input_arc, 0, const_item_len, None).unwrap();
        let mut parser = Parser::new(handler, &token_stream);
        // perform the parse
        let const_item: ItemConst = parser.parse()?;
        let const_item_span = const_item.span();

        // perform the conversions from parser code to parse tree types
        let name = const_item.name.clone();
        let attributes = Default::default();
        // convert to const decl
        let const_decl_id = to_parsed_lang::item_const_to_constant_declaration(
            &mut to_parsed_lang::Context::new(crate::BuildTarget::EVM, experimental),
            handler,
            engines,
            const_item,
            attributes,
            true,
        )?;

        // Temporarily disallow non-literals. See https://github.com/FuelLabs/sway/issues/2647.
        let const_decl = engines.pe().get_constant(&const_decl_id);
        let has_literal = match &const_decl.value {
            Some(value) => {
                matches!(value.kind, ExpressionKind::Literal(_))
            }
            None => false,
        };

        if !has_literal {
            return Err(handler.emit_err(CompileError::ContractIdValueNotALiteral {
                span: const_item_span,
            }));
        }

        let ast_node = AstNode {
            content: AstNodeContent::Declaration(Declaration::ConstantDeclaration(const_decl_id)),
            span: const_item_span.clone(),
        };
        let root = Root::from(Module::default());
        let mut ns = Namespace::init_root(root);
        // This is pretty hacky but that's okay because of this code is being removed pretty soon
        ns.root.module.name = ns_name;
        ns.root.module.is_external = true;
        ns.root.module.visibility = Visibility::Public;
        let type_check_ctx = TypeCheckContext::from_namespace(&mut ns, engines, experimental);
        let typed_node = ty::TyAstNode::type_check(handler, type_check_ctx, ast_node).unwrap();
        // get the decl out of the typed node:
        // we know as an invariant this must be a const decl, as we hardcoded a const decl in
        // the above `format!`.  if it isn't we report an
        // error that only constant items are allowed, defensive programming etc...
        let typed_decl = match typed_node.content {
            ty::TyAstNodeContent::Declaration(decl) => decl,
            _ => {
                return Err(
                    handler.emit_err(CompileError::ContractIdConstantNotAConstDecl {
                        span: const_item_span,
                    }),
                );
            }
        };
        compiled_constants.insert(name, typed_decl);

        let mut ret = Self::default();
        ret.current_lexical_scope_mut().items.symbols = compiled_constants;
        Ok(ret)
    }

    /// Immutable access to this module's submodules.
    pub fn submodules(&self) -> &im::OrdMap<ModuleName, Module> {
        &self.submodules
    }

    /// Insert a submodule into this `Module`.
    pub fn insert_submodule(&mut self, name: String, submodule: Module) {
        self.submodules.insert(name, submodule);
    }

    /// Lookup the submodule at the given path.
    pub fn submodule(&self, _engines: &Engines, path: &ModulePath) -> Option<&Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Unique access to the submodule at the given path.
    pub fn submodule_mut(&mut self, _engines: &Engines, path: &ModulePath) -> Option<&mut Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get_mut(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Lookup the submodule at the given path.
    ///
    /// This should be used rather than `Index` when we don't yet know whether the module exists.
    pub(crate) fn lookup_submodule(
        &self,
        handler: &Handler,
        engines: &Engines,
        path: &[Ident],
    ) -> Result<&Module, ErrorEmitted> {
        match self.submodule(engines, path) {
            None => Err(handler.emit_err(module_not_found(path))),
            Some(module) => Ok(module),
        }
    }

    /// Lookup the submodule at the given path.
    ///
    /// This should be used rather than `Index` when we don't yet know whether the module exists.
    pub(crate) fn lookup_submodule_mut(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        path: &[Ident],
    ) -> Result<&mut Module, ErrorEmitted> {
        match self.submodule_mut(engines, path) {
            None => Err(handler.emit_err(module_not_found(path))),
            Some(module) => Ok(module),
        }
    }

    /// Returns the current lexical scope associated with this module.
    fn current_lexical_scope(&self) -> &LexicalScope {
        self.lexical_scopes
            .get(self.current_lexical_scope_id)
            .unwrap()
    }

    /// Returns the mutable current lexical scope associated with this module.
    fn current_lexical_scope_mut(&mut self) -> &mut LexicalScope {
        self.lexical_scopes
            .get_mut(self.current_lexical_scope_id)
            .unwrap()
    }

    /// The collection of items declared by this module's current lexical scope.
    pub fn current_items(&self) -> &Items {
        &self.current_lexical_scope().items
    }

    /// The mutable collection of items declared by this module's curent lexical scope.
    pub fn current_items_mut(&mut self) -> &mut Items {
        &mut self.current_lexical_scope_mut().items
    }

    pub fn current_lexical_scope_id(&self) -> LexicalScopeId {
        self.current_lexical_scope_id
    }

    /// Pushes a new scope to the module's lexical scope hierarchy.
    pub fn push_new_lexical_scope(&mut self) -> LexicalScopeId {
        let previous_scope_id = self.current_lexical_scope_id();
        let new_scoped_id = {
            self.lexical_scopes.push(LexicalScope {
                parent: Some(previous_scope_id),
                ..Default::default()
            });
            self.current_lexical_scope_id()
        };
        let previous_scope = self.lexical_scopes.get_mut(previous_scope_id).unwrap();
        previous_scope.children.push(new_scoped_id);
        self.current_lexical_scope_id = new_scoped_id;
        new_scoped_id
    }

    /// Pops the current scope from the module's lexical scope hierarchy.
    pub fn pop_lexical_scope(&mut self) {
        let parent_scope_id = self.current_lexical_scope().parent;
        self.current_lexical_scope_id = parent_scope_id.unwrap_or(0);
    }
}

impl From<Root> for Module {
    fn from(root: Root) -> Self {
        root.module
    }
}

fn module_not_found(path: &[Ident]) -> CompileError {
    CompileError::ModuleNotFound {
        span: path.iter().fold(path[0].span(), |acc, this_one| {
            if acc.source_id() == this_one.span().source_id() {
                Span::join(acc, this_one.span())
            } else {
                acc
            }
        }),
        name: path
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("::"),
    }
}
