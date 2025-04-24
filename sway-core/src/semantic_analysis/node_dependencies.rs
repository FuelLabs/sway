use crate::{
    decl_engine::ParsedDeclEngineGet,
    language::{parsed::*, CallPath},
    type_system::*,
    Engines,
};
use hashbrown::{HashMap, HashSet};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    iter::FromIterator,
};
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::integer_bits::IntegerBits;
use sway_types::Spanned;
use sway_types::{ident::Ident, span::Span};

// -------------------------------------------------------------------------------------------------
/// Take a list of nodes and reorder them so that they may be semantically analysed without any
/// dependencies breaking.
pub(crate) fn order_ast_nodes_by_dependency(
    handler: &Handler,
    engines: &Engines,
    nodes: Vec<AstNode>,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let decl_dependencies = DependencyMap::from_iter(
        nodes
            .iter()
            .filter_map(|node| Dependencies::gather_from_decl_node(engines, node)),
    );

    // Check here for recursive calls now that we have a nice map of the dependencies to help us.
    let mut errors = find_recursive_decls(&decl_dependencies);

    handler.scope(|handler| {
        // Because we're pulling these errors out of a HashMap they'll probably be in a funny
        // order.  Here we'll sort them by span start.
        errors.sort_by_key(|err| err.span().start());

        for err in errors {
            handler.emit_err(err);
        }
        Ok(())
    })?;

    // Reorder the parsed AstNodes based on dependency. Includes first, then uses, then
    // reordered declarations, then anything else.  To keep the list stable and simple we can
    // use a basic insertion sort.
    Ok(nodes
        .into_iter()
        .fold(Vec::<AstNode>::new(), |ordered, node| {
            insert_into_ordered_nodes(engines, &decl_dependencies, ordered, node)
        }))
}

// -------------------------------------------------------------------------------------------------
// Recursion detection.

fn find_recursive_decls(decl_dependencies: &DependencyMap) -> Vec<CompileError> {
    decl_dependencies
        .iter()
        .filter_map(|(dep_sym, _)| find_recursive_decl(decl_dependencies, dep_sym))
        .collect()
}

fn find_recursive_decl(
    decl_dependencies: &DependencyMap,
    dep_sym: &DependentSymbol,
) -> Option<CompileError> {
    match dep_sym {
        DependentSymbol::Fn(_, _, Some(fn_span)) => {
            let mut chain = Vec::new();
            find_recursive_call_chain(decl_dependencies, dep_sym, fn_span, &mut chain)
        }
        DependentSymbol::Symbol(_, _) => {
            let mut chain = Vec::new();
            find_recursive_type_chain(decl_dependencies, dep_sym, &mut chain)
        }
        _otherwise => None,
    }
}

fn find_recursive_call_chain(
    decl_dependencies: &DependencyMap,
    fn_sym: &DependentSymbol,
    fn_span: &Span,
    chain: &mut Vec<Ident>,
) -> Option<CompileError> {
    if let DependentSymbol::Fn(_, fn_sym_ident, _) = fn_sym {
        if chain.contains(fn_sym_ident) {
            // We've found a recursive loop, but it's possible this function is not actually in the
            // loop, but is instead just calling into the loop.  Only if this function is at the
            // start of the chain do we need to report it.
            return if &chain[0] != fn_sym_ident {
                None
            } else {
                Some(build_recursion_error(
                    fn_sym_ident.clone(),
                    fn_span.clone(),
                    &chain[1..],
                ))
            };
        }
        decl_dependencies.get(fn_sym).and_then(|deps_set| {
            chain.push(fn_sym_ident.clone());
            let result = deps_set.deps.iter().find_map(|dep_sym| {
                find_recursive_call_chain(decl_dependencies, dep_sym, fn_span, chain)
            });
            chain.pop();
            result
        })
    } else {
        None
    }
}

fn find_recursive_type_chain(
    decl_dependencies: &DependencyMap,
    dep_sym: &DependentSymbol,
    chain: &mut Vec<Ident>,
) -> Option<CompileError> {
    if let DependentSymbol::Symbol(_, sym_ident) = dep_sym {
        if chain.contains(sym_ident) {
            // See above about it only being an error if we're referring back to the start.
            return if &chain[0] != sym_ident {
                None
            } else {
                Some(build_recursive_type_error(sym_ident.clone(), &chain[1..]))
            };
        }
        decl_dependencies.get(dep_sym).and_then(|deps_set| {
            chain.push(sym_ident.clone());
            let result = deps_set
                .deps
                .iter()
                .find_map(|dep_sym| find_recursive_type_chain(decl_dependencies, dep_sym, chain));
            chain.pop();
            result
        })
    } else {
        None
    }
}

fn build_recursion_error(fn_sym: Ident, span: Span, chain: &[Ident]) -> CompileError {
    match chain.len() {
        // An empty chain indicates immediate recursion.
        0 => CompileError::RecursiveCall {
            fn_name: fn_sym,
            span,
        },
        // Chain entries indicate mutual recursion.
        1 => CompileError::RecursiveCallChain {
            fn_name: fn_sym,
            call_chain: chain[0].as_str().to_string(),
            span,
        },
        n => {
            let mut msg = chain[0].as_str().to_string();
            for ident in &chain[1..(n - 1)] {
                msg.push_str(", ");
                msg.push_str(ident.as_str());
            }
            msg.push_str(" and ");
            msg.push_str(chain[n - 1].as_str());
            CompileError::RecursiveCallChain {
                fn_name: fn_sym,
                call_chain: msg,
                span,
            }
        }
    }
}

fn build_recursive_type_error(name: Ident, chain: &[Ident]) -> CompileError {
    let span = name.span();
    match chain.len() {
        // An empty chain indicates immediate recursion.
        0 => CompileError::RecursiveType { name, span },
        // Chain entries indicate mutual recursion.
        1 => CompileError::RecursiveTypeChain {
            name,
            type_chain: chain[0].as_str().to_string(),
            span,
        },
        n => {
            let mut msg = chain[0].as_str().to_string();
            for ident in &chain[1..(n - 1)] {
                msg.push_str(", ");
                msg.push_str(ident.as_str());
            }
            msg.push_str(" and ");
            msg.push_str(chain[n - 1].as_str());
            CompileError::RecursiveTypeChain {
                name,
                type_chain: msg,
                span,
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Dependency gathering.

#[derive(Default)]
struct MemoizedBuildHasher {}

impl std::hash::BuildHasher for MemoizedBuildHasher {
    type Hasher = MemoizedHasher;

    fn build_hasher(&self) -> Self::Hasher {
        MemoizedHasher { last_u64: None }
    }
}

// Only works with `write_u64`, because it returns the last "hashed" u64, as is.
struct MemoizedHasher {
    last_u64: Option<u64>,
}

impl std::hash::Hasher for MemoizedHasher {
    fn finish(&self) -> u64 {
        *self.last_u64.as_ref().unwrap()
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!("Only works with write_u64");
    }

    fn write_u64(&mut self, i: u64) {
        self.last_u64 = Some(i);
    }
}

type DependencyMap = HashMap<DependentSymbol, Dependencies, MemoizedBuildHasher>;
type DependencySet = HashSet<DependentSymbol, MemoizedBuildHasher>;

fn insert_into_ordered_nodes(
    engines: &Engines,
    decl_dependencies: &DependencyMap,
    mut ordered_nodes: Vec<AstNode>,
    node: AstNode,
) -> Vec<AstNode> {
    for idx in 0..ordered_nodes.len() {
        // If we find a node which depends on the new node, insert it in front.
        if depends_on(engines, decl_dependencies, &ordered_nodes[idx], &node) {
            ordered_nodes.insert(idx, node);
            return ordered_nodes;
        }
    }

    // Node wasn't inserted into list, append it now.
    ordered_nodes.push(node);
    ordered_nodes
}

// dependant: noun; thing depending on another thing.
// dependee: noun; thing which is depended upon by another thing.
//
// Does the dependant depend on the dependee?

fn depends_on(
    engines: &Engines,
    decl_dependencies: &DependencyMap,
    dependant_node: &AstNode,
    dependee_node: &AstNode,
) -> bool {
    match (&dependant_node.content, &dependee_node.content) {
        // Include statements first.
        (AstNodeContent::IncludeStatement(_), AstNodeContent::IncludeStatement(_)) => false,
        (_, AstNodeContent::IncludeStatement(_)) => true,

        // Use statements next.
        (AstNodeContent::IncludeStatement(_), AstNodeContent::UseStatement(_)) => false,
        (AstNodeContent::UseStatement(_), AstNodeContent::UseStatement(_)) => false,
        (_, AstNodeContent::UseStatement(_)) => true,

        // Then declarations, ordered using the dependencies list.
        (AstNodeContent::IncludeStatement(_), AstNodeContent::Declaration(_)) => false,
        (AstNodeContent::UseStatement(_), AstNodeContent::Declaration(_)) => false,
        (AstNodeContent::Declaration(dependant), AstNodeContent::Declaration(dependee)) => {
            match (decl_name(engines, dependant), decl_name(engines, dependee)) {
                (Some(dependant_name), Some(dependee_name)) => decl_dependencies
                    .get(&dependant_name)
                    .map(|deps_set| {
                        recursively_depends_on(&deps_set.deps, &dependee_name, decl_dependencies)
                    })
                    .unwrap_or(false),
                _ => false,
            }
        }
        (_, AstNodeContent::Declaration(_)) => true,

        // Everything else we don't care.
        _ => false,
    }
}

// -------------------------------------------------------------------------------------------------
// Dependencies are just a collection of dependee symbols.

#[derive(Debug)]
struct Dependencies {
    deps: DependencySet,
}

impl Dependencies {
    fn gather_from_decl_node(
        engines: &Engines,
        node: &AstNode,
    ) -> Option<(DependentSymbol, Dependencies)> {
        match &node.content {
            AstNodeContent::Declaration(decl) => decl_name(engines, decl).map(|name| {
                (
                    name,
                    Dependencies {
                        deps: DependencySet::default(),
                    }
                    .gather_from_decl(engines, decl),
                )
            }),
            _ => None,
        }
    }

    fn gather_from_decl(self, engines: &Engines, decl: &Declaration) -> Self {
        match decl {
            Declaration::VariableDeclaration(decl_id) => {
                let VariableDeclaration {
                    type_ascription,
                    body,
                    ..
                } = &*engines.pe().get_variable(decl_id);
                self.gather_from_type_argument(engines, type_ascription)
                    .gather_from_expr(engines, body)
            }
            Declaration::ConstantDeclaration(decl_id) => {
                let decl = engines.pe().get_constant(decl_id);
                self.gather_from_constant_decl(engines, &decl)
            }
            Declaration::ConfigurableDeclaration(decl_id) => {
                let decl = engines.pe().get_configurable(decl_id);
                self.gather_from_configurable_decl(engines, &decl)
            }
            Declaration::TraitTypeDeclaration(decl_id) => {
                let decl = engines.pe().get_trait_type(decl_id);
                self.gather_from_type_decl(engines, &decl)
            }
            Declaration::TraitFnDeclaration(decl_id) => {
                let decl = engines.pe().get_trait_fn(decl_id);
                self.gather_from_trait_fn_decl(engines, &decl)
            }
            Declaration::FunctionDeclaration(decl_id) => {
                let fn_decl = engines.pe().get_function(decl_id);
                self.gather_from_fn_decl(engines, &fn_decl)
            }
            Declaration::StructDeclaration(decl_id) => {
                let StructDeclaration {
                    fields,
                    type_parameters,
                    ..
                } = &*engines.pe().get_struct(decl_id);
                self.gather_from_iter(fields.iter(), |deps, field| {
                    deps.gather_from_type_argument(engines, &field.type_argument)
                })
                .gather_from_type_parameters(type_parameters)
            }
            Declaration::EnumDeclaration(decl_id) => {
                let EnumDeclaration {
                    variants,
                    type_parameters,
                    ..
                } = &*engines.pe().get_enum(decl_id);
                self.gather_from_iter(variants.iter(), |deps, variant| {
                    deps.gather_from_type_argument(engines, &variant.type_argument)
                })
                .gather_from_type_parameters(type_parameters)
            }
            Declaration::EnumVariantDeclaration(_decl) => unreachable!(),
            Declaration::TraitDeclaration(decl_id) => {
                let trait_decl = engines.pe().get_trait(decl_id);
                self.gather_from_iter(trait_decl.supertraits.iter(), |deps, sup| {
                    deps.gather_from_call_path(&sup.name, false, false)
                })
                .gather_from_iter(
                    trait_decl.interface_surface.iter(),
                    |deps, item| match item {
                        TraitItem::TraitFn(decl_id) => {
                            let sig = engines.pe().get_trait_fn(decl_id);
                            deps.gather_from_iter(sig.parameters.iter(), |deps, param| {
                                deps.gather_from_type_argument(engines, &param.type_argument)
                            })
                            .gather_from_type_argument(engines, &sig.return_type)
                        }
                        TraitItem::Constant(decl_id) => {
                            let const_decl = engines.pe().get_constant(decl_id);
                            deps.gather_from_constant_decl(engines, &const_decl)
                        }
                        TraitItem::Type(decl_id) => {
                            let type_decl = engines.pe().get_trait_type(decl_id);
                            deps.gather_from_type_decl(engines, &type_decl)
                        }
                        TraitItem::Error(_, _) => deps,
                    },
                )
                .gather_from_iter(
                    trait_decl.methods.iter(),
                    |deps, fn_decl_id| {
                        let fn_decl = engines.pe().get_function(fn_decl_id);
                        deps.gather_from_fn_decl(engines, &fn_decl)
                    },
                )
            }
            Declaration::ImplSelfOrTrait(decl_id) => {
                let ImplSelfOrTrait {
                    impl_type_parameters,
                    trait_name,
                    implementing_for,
                    items,
                    ..
                } = &*engines.pe().get_impl_self_or_trait(decl_id);
                self.gather_from_call_path(trait_name, false, false)
                    .gather_from_type_argument(engines, implementing_for)
                    .gather_from_type_parameters(impl_type_parameters)
                    .gather_from_iter(items.iter(), |deps, item| match item {
                        ImplItem::Fn(fn_decl_id) => {
                            let fn_decl = engines.pe().get_function(fn_decl_id);
                            deps.gather_from_fn_decl(engines, &fn_decl)
                        }
                        ImplItem::Constant(decl_id) => {
                            let const_decl = engines.pe().get_constant(decl_id);
                            deps.gather_from_constant_decl(engines, &const_decl)
                        }
                        ImplItem::Type(decl_id) => {
                            let type_decl = engines.pe().get_trait_type(decl_id);
                            deps.gather_from_type_decl(engines, &type_decl)
                        }
                    })
            }
            Declaration::AbiDeclaration(decl_id) => {
                let AbiDeclaration {
                    interface_surface,
                    methods,
                    supertraits,
                    ..
                } = &*engines.pe().get_abi(decl_id);

                self.gather_from_iter(supertraits.iter(), |deps, sup| {
                    deps.gather_from_call_path(&sup.name, false, false)
                })
                .gather_from_iter(interface_surface.iter(), |deps, item| match item {
                    TraitItem::TraitFn(decl_id) => {
                        let sig = engines.pe().get_trait_fn(decl_id);
                        deps.gather_from_iter(sig.parameters.iter(), |deps, param| {
                            deps.gather_from_type_argument(engines, &param.type_argument)
                        })
                        .gather_from_type_argument(engines, &sig.return_type)
                    }
                    TraitItem::Constant(decl_id) => {
                        let const_decl = engines.pe().get_constant(decl_id);
                        deps.gather_from_constant_decl(engines, &const_decl)
                    }
                    TraitItem::Type(decl_id) => {
                        let type_decl = engines.pe().get_trait_type(decl_id);
                        deps.gather_from_type_decl(engines, &type_decl)
                    }
                    TraitItem::Error(_, _) => deps,
                })
                .gather_from_iter(methods.iter(), |deps, fn_decl_id| {
                    let fn_decl = engines.pe().get_function(fn_decl_id);
                    deps.gather_from_fn_decl(engines, &fn_decl)
                })
            }
            Declaration::StorageDeclaration(decl_id) => {
                let StorageDeclaration { entries, .. } = &*engines.pe().get_storage(decl_id);
                self.gather_from_iter(entries.iter(), |deps, entry| {
                    deps.gather_from_storage_entry(engines, entry)
                })
            }
            Declaration::TypeAliasDeclaration(decl_id) => {
                let TypeAliasDeclaration { ty, .. } = &*engines.pe().get_type_alias(decl_id);
                self.gather_from_type_argument(engines, ty)
            }
            Declaration::ConstGenericDeclaration(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }

    fn gather_from_storage_entry(self, engines: &Engines, entry: &StorageEntry) -> Self {
        match entry {
            StorageEntry::Namespace(namespace) => self
                .gather_from_iter(namespace.entries.iter(), |deps, entry| {
                    deps.gather_from_storage_entry(engines, entry)
                }),
            StorageEntry::Field(field) => {
                self.gather_from_type_argument(engines, &field.type_argument)
            }
        }
    }

    fn gather_from_constant_decl(
        self,
        engines: &Engines,
        const_decl: &ConstantDeclaration,
    ) -> Self {
        let ConstantDeclaration {
            type_ascription,
            value,
            ..
        } = const_decl;
        match value {
            Some(value) => self
                .gather_from_type_argument(engines, type_ascription)
                .gather_from_expr(engines, value),
            None => self,
        }
    }

    fn gather_from_configurable_decl(
        self,
        engines: &Engines,
        const_decl: &ConfigurableDeclaration,
    ) -> Self {
        let ConfigurableDeclaration {
            type_ascription,
            value,
            ..
        } = const_decl;
        match value {
            Some(value) => self
                .gather_from_type_argument(engines, type_ascription)
                .gather_from_expr(engines, value),
            None => self,
        }
    }

    fn gather_from_type_decl(self, engines: &Engines, type_decl: &TraitTypeDeclaration) -> Self {
        let TraitTypeDeclaration { ty_opt, .. } = type_decl;
        match ty_opt {
            Some(value) => self.gather_from_type_argument(engines, value),
            None => self,
        }
    }

    fn gather_from_trait_fn_decl(self, engines: &Engines, fn_decl: &TraitFn) -> Self {
        let TraitFn {
            parameters,
            return_type,
            ..
        } = fn_decl;
        self.gather_from_iter(parameters.iter(), |deps, param| {
            deps.gather_from_type_argument(engines, &param.type_argument)
        })
        .gather_from_type_argument(engines, return_type)
    }

    fn gather_from_fn_decl(self, engines: &Engines, fn_decl: &FunctionDeclaration) -> Self {
        let FunctionDeclaration {
            parameters,
            return_type,
            body,
            type_parameters,
            ..
        } = fn_decl;
        self.gather_from_iter(parameters.iter(), |deps, param| {
            deps.gather_from_type_argument(engines, &param.type_argument)
        })
        .gather_from_type_argument(engines, return_type)
        .gather_from_block(engines, body)
        .gather_from_type_parameters(type_parameters)
    }

    fn gather_from_expr(self, engines: &Engines, expr: &Expression) -> Self {
        match &expr.kind {
            ExpressionKind::Variable(name) => {
                // in the case of ABI variables, we actually want to check if the ABI needs to be
                // ordered
                self.gather_from_call_path(&(name.clone()).into(), false, false)
            }
            ExpressionKind::AmbiguousVariableExpression(name) => {
                self.gather_from_call_path(&(name.clone()).into(), false, false)
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                let FunctionApplicationExpression {
                    call_path_binding,
                    resolved_call_path_binding: _,
                    arguments,
                } = &**function_application_expression;
                self.gather_from_call_path(&call_path_binding.inner, false, true)
                    .gather_from_type_arguments(engines, &call_path_binding.type_arguments.to_vec())
                    .gather_from_iter(arguments.iter(), |deps, arg| {
                        deps.gather_from_expr(engines, arg)
                    })
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => self
                .gather_from_expr(engines, lhs)
                .gather_from_expr(engines, rhs),
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
                ..
            }) => if let Some(else_expr) = r#else {
                self.gather_from_expr(engines, else_expr)
            } else {
                self
            }
            .gather_from_expr(engines, condition)
            .gather_from_expr(engines, then),
            ExpressionKind::Match(MatchExpression {
                value, branches, ..
            }) => self
                .gather_from_expr(engines, value)
                .gather_from_iter(branches.iter(), |deps, branch| {
                    deps.gather_from_match_branch(engines, branch)
                }),
            ExpressionKind::CodeBlock(contents) => self.gather_from_block(engines, contents),
            ExpressionKind::Array(ArrayExpression::Explicit { contents, .. }) => self
                .gather_from_iter(contents.iter(), |deps, expr| {
                    deps.gather_from_expr(engines, expr)
                }),
            ExpressionKind::Array(ArrayExpression::Repeat { value, length }) => self
                .gather_from_expr(engines, value)
                .gather_from_expr(engines, length),
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index, .. }) => self
                .gather_from_expr(engines, prefix)
                .gather_from_expr(engines, index),
            ExpressionKind::Struct(struct_expression) => {
                let StructExpression {
                    call_path_binding,
                    resolved_call_path_binding: _,
                    fields,
                } = &**struct_expression;
                self.gather_from_call_path(&call_path_binding.inner, false, false)
                    .gather_from_type_arguments(engines, &call_path_binding.type_arguments.to_vec())
                    .gather_from_iter(fields.iter(), |deps, field| {
                        deps.gather_from_expr(engines, &field.value)
                    })
            }
            ExpressionKind::Subfield(SubfieldExpression { prefix, .. }) => {
                self.gather_from_expr(engines, prefix)
            }
            ExpressionKind::AmbiguousPathExpression(e) => {
                let AmbiguousPathExpression {
                    call_path_binding,
                    args,
                    qualified_path_root: _,
                } = &**e;
                let mut this = self;
                if call_path_binding.inner.prefixes.is_empty() {
                    if let Some(before) = &call_path_binding.inner.suffix.before {
                        // We have just `Foo::Bar`, and nothing before `Foo`,
                        // so this could be referring to `Enum::Variant`,
                        // so we want to depend on `Enum` but not `Variant`.
                        this.deps
                            .insert(DependentSymbol::new_symbol(before.inner.clone()));
                    } else {
                        // We have just `Foo`, and nothing before `Foo`,
                        // so this is could either an enum variant or a function application
                        // so we want to depend on it as a function
                        this.deps.insert(DependentSymbol::new_fn(
                            call_path_binding.inner.suffix.suffix.clone(),
                            None,
                        ));
                    }
                }
                this.gather_from_type_arguments(engines, &call_path_binding.type_arguments.to_vec())
                    .gather_from_iter(args.iter(), |deps, arg| deps.gather_from_expr(engines, arg))
            }
            ExpressionKind::DelineatedPath(delineated_path_expression) => {
                let DelineatedPathExpression {
                    call_path_binding,
                    args,
                } = &**delineated_path_expression;
                // It's either a module path which we can ignore, or an enum variant path, in which
                // case we're interested in the enum name and initialiser args, ignoring the
                // variant name.
                let args_vec = args.clone().unwrap_or_default();
                self.gather_from_call_path(&call_path_binding.inner.call_path, true, false)
                    .gather_from_type_arguments(engines, &call_path_binding.type_arguments.to_vec())
                    .gather_from_iter(args_vec.iter(), |deps, arg| {
                        deps.gather_from_expr(engines, arg)
                    })
            }
            ExpressionKind::MethodApplication(method_application_expression) => self
                .gather_from_iter(
                    method_application_expression.arguments.iter(),
                    |deps, arg| deps.gather_from_expr(engines, arg),
                ),
            ExpressionKind::Asm(asm) => self
                .gather_from_iter(asm.registers.iter(), |deps, register| {
                    deps.gather_from_opt_expr(engines, register.initializer.as_ref())
                })
                .gather_from_typeinfo(engines, &asm.return_type),

            // we should do address someday, but due to the whole `re_parse_expression` thing
            // it isn't possible right now
            ExpressionKind::AbiCast(abi_cast_expression) => {
                self.gather_from_call_path(&abi_cast_expression.abi_name, false, false)
            }

            ExpressionKind::Literal(_)
            | ExpressionKind::Break
            | ExpressionKind::Continue
            | ExpressionKind::StorageAccess(_)
            | ExpressionKind::Error(_, _) => self,

            ExpressionKind::Tuple(fields) => self.gather_from_iter(fields.iter(), |deps, field| {
                deps.gather_from_expr(engines, field)
            }),
            ExpressionKind::TupleIndex(TupleIndexExpression { prefix, .. }) => {
                self.gather_from_expr(engines, prefix)
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                arguments, ..
            }) => self.gather_from_iter(arguments.iter(), |deps, arg| {
                deps.gather_from_expr(engines, arg)
            }),
            ExpressionKind::WhileLoop(WhileLoopExpression {
                condition, body, ..
            }) => self
                .gather_from_expr(engines, condition)
                .gather_from_block(engines, body),
            ExpressionKind::ForLoop(ForLoopExpression { desugared, .. }) => {
                self.gather_from_expr(engines, desugared)
            }
            ExpressionKind::Reassignment(reassignment) => {
                self.gather_from_expr(engines, &reassignment.rhs)
            }
            ExpressionKind::ImplicitReturn(expr) | ExpressionKind::Return(expr) => {
                self.gather_from_expr(engines, expr)
            }
            ExpressionKind::Ref(RefExpression { value: expr, .. })
            | ExpressionKind::Deref(expr) => self.gather_from_expr(engines, expr),
        }
    }

    fn gather_from_match_branch(self, engines: &Engines, branch: &MatchBranch) -> Self {
        let MatchBranch {
            scrutinee, result, ..
        } = branch;
        self.gather_from_iter(
            scrutinee.gather_approximate_typeinfo_dependencies().iter(),
            |deps, type_info| deps.gather_from_typeinfo(engines, type_info),
        )
        .gather_from_expr(engines, result)
    }

    fn gather_from_opt_expr(self, engines: &Engines, opt_expr: Option<&Expression>) -> Self {
        match opt_expr {
            None => self,
            Some(expr) => self.gather_from_expr(engines, expr),
        }
    }

    fn gather_from_block(self, engines: &Engines, block: &CodeBlock) -> Self {
        self.gather_from_iter(block.contents.iter(), |deps, node| {
            deps.gather_from_node(engines, node)
        })
    }

    fn gather_from_node(self, engines: &Engines, node: &AstNode) -> Self {
        match &node.content {
            AstNodeContent::Expression(expr) => self.gather_from_expr(engines, expr),
            AstNodeContent::Declaration(decl) => self.gather_from_decl(engines, decl),

            // No deps from these guys.
            AstNodeContent::UseStatement(_)
            | AstNodeContent::IncludeStatement(_)
            | AstNodeContent::Error(_, _) => self,
        }
    }

    fn gather_from_call_path(
        mut self,
        call_path: &CallPath,
        use_prefix: bool,
        is_fn_app: bool,
    ) -> Self {
        if call_path.prefixes.is_empty() {
            // We can just use the suffix.
            self.deps.insert(if is_fn_app {
                DependentSymbol::new_fn(call_path.suffix.clone(), None)
            } else {
                DependentSymbol::new_symbol(call_path.suffix.clone())
            });
        } else if use_prefix && call_path.prefixes.len() == 1 {
            // Here we can use the prefix (e.g., for 'Enum::Variant' -> 'Enum') as long is it's
            // only a single element.
            self.deps
                .insert(DependentSymbol::new_symbol(call_path.prefixes[0].clone()));
        }
        self
    }

    fn gather_from_type_parameters(self, type_parameters: &[TypeParameter]) -> Self {
        self.gather_from_iter(type_parameters.iter(), |deps, p| match p {
            TypeParameter::Type(p) => deps
                .gather_from_iter(p.trait_constraints.iter(), |deps, constraint| {
                    deps.gather_from_call_path(&constraint.trait_name, false, false)
                }),
            TypeParameter::Const(_) => deps,
        })
    }

    fn gather_from_type_arguments(
        self,
        engines: &Engines,
        type_arguments: &[GenericArgument],
    ) -> Self {
        self.gather_from_iter(type_arguments.iter(), |deps, type_argument| {
            deps.gather_from_type_argument(engines, type_argument)
        })
    }

    fn gather_from_type_argument(self, engines: &Engines, type_argument: &GenericArgument) -> Self {
        let type_engine = engines.te();
        self.gather_from_typeinfo(engines, &type_engine.get(type_argument.type_id()))
    }

    fn gather_from_typeinfo(mut self, engines: &Engines, type_info: &TypeInfo) -> Self {
        let decl_engine = engines.de();
        match type_info {
            TypeInfo::ContractCaller {
                abi_name: AbiName::Known(abi_name),
                ..
            } => self.gather_from_call_path(abi_name, false, false),
            TypeInfo::Custom {
                qualified_call_path: name,
                type_arguments,
            } => {
                self.deps
                    .insert(DependentSymbol::new_symbol(name.clone().call_path.suffix));
                match type_arguments {
                    Some(type_arguments) => {
                        self.gather_from_type_arguments(engines, type_arguments)
                    }
                    None => self,
                }
            }
            TypeInfo::Tuple(elems) => self.gather_from_iter(elems.iter(), |deps, elem| {
                deps.gather_from_type_argument(engines, elem)
            }),
            TypeInfo::Array(elem_type, _) => self.gather_from_type_argument(engines, elem_type),
            TypeInfo::Slice(elem_type) => self.gather_from_type_argument(engines, elem_type),
            TypeInfo::Struct(decl_ref) => self.gather_from_iter(
                decl_engine.get_struct(decl_ref).fields.iter(),
                |deps, field| deps.gather_from_type_argument(engines, &field.type_argument),
            ),
            TypeInfo::Enum(decl_ref) => self.gather_from_iter(
                decl_engine.get_enum(decl_ref).variants.iter(),
                |deps, variant| deps.gather_from_type_argument(engines, &variant.type_argument),
            ),
            TypeInfo::Alias { ty, .. } => self.gather_from_type_argument(engines, ty),
            _ => self,
        }
    }

    fn gather_from_iter<I: Iterator, F: FnMut(Self, I::Item) -> Self>(self, iter: I, f: F) -> Self {
        iter.fold(self, f)
    }
}

// -------------------------------------------------------------------------------------------------
// Most declarations can be uniquely identified by a name str.  ImplSelf and ImplTrait don't have a
// name of their own though.  They can be identified as being an impl of another type, so we make
// the distinction here with DependentSymbol.
//
// At the same time, we don't need to identify ImplSelf and ImplTrait as dependencies, as while
// they themselves depend on other declarations, no declarations depend on them.  This is
// illustrated in DependentSymbol::is().

#[derive(Debug, Eq)]
enum DependentSymbol {
    Symbol(u64, Ident),
    Fn(u64, Ident, Option<Span>),
    Impl(u64),
}

impl Hash for DependentSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cached_hash().hash(state);
    }
}

impl DependentSymbol {
    pub fn new_symbol(name: Ident) -> Self {
        let mut hasher = DefaultHasher::new();
        0.hash(&mut hasher);
        name.hash(&mut hasher);
        Self::Symbol(hasher.finish(), name)
    }

    pub fn new_fn(name: Ident, span: Option<Span>) -> Self {
        let mut hasher = DefaultHasher::new();
        1.hash(&mut hasher);
        name.hash(&mut hasher);
        // TODO span?
        Self::Fn(hasher.finish(), name, span)
    }

    pub fn new_impl(name: Ident, impl_for: String, method_names: String) -> Self {
        let mut hasher = DefaultHasher::new();
        2.hash(&mut hasher);
        name.hash(&mut hasher);
        impl_for.hash(&mut hasher);
        method_names.hash(&mut hasher);
        Self::Impl(hasher.finish())
    }

    pub fn cached_hash(&self) -> u64 {
        match self {
            DependentSymbol::Symbol(hash, ..) => *hash,
            DependentSymbol::Fn(hash, ..) => *hash,
            DependentSymbol::Impl(hash, ..) => *hash,
        }
    }
}

impl PartialEq for DependentSymbol {
    fn eq(&self, rhs: &Self) -> bool {
        self.cached_hash().eq(&rhs.cached_hash())
    }
}

fn decl_name(engines: &Engines, decl: &Declaration) -> Option<DependentSymbol> {
    let type_engine = engines.te();
    let dep_sym = |name| Some(DependentSymbol::new_symbol(name));

    // `method_names` is the concatenation of all the method names defined in an impl block.
    // This is needed because there can exist multiple impl self blocks for a single type in a
    // file and we need some way to disambiguate them.
    let impl_sym = |trait_name, type_info: &TypeInfo, method_names| {
        Some(DependentSymbol::new_impl(
            trait_name,
            type_info_name(type_info),
            method_names,
        ))
    };

    match decl {
        // These declarations can depend upon other declarations.
        Declaration::FunctionDeclaration(decl_id) => {
            let decl = engines.pe().get_function(decl_id);
            Some(DependentSymbol::new_fn(
                decl.name.clone(),
                Some(decl.span.clone()),
            ))
        }
        Declaration::ConstantDeclaration(decl_id) => {
            let decl = engines.pe().get_constant(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::ConfigurableDeclaration(decl_id) => {
            let decl = engines.pe().get_configurable(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::TraitTypeDeclaration(decl_id) => {
            let decl = engines.pe().get_trait_type(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::TraitFnDeclaration(decl_id) => {
            let decl = engines.pe().get_trait_fn(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::StructDeclaration(decl_id) => {
            let decl = engines.pe().get_struct(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::EnumDeclaration(decl_id) => {
            let decl = engines.pe().get_enum(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::EnumVariantDeclaration(_decl) => None,
        Declaration::TraitDeclaration(decl_id) => {
            let decl = engines.pe().get_trait(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::AbiDeclaration(decl_id) => {
            let decl = engines.pe().get_abi(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::TypeAliasDeclaration(decl_id) => {
            let decl = engines.pe().get_type_alias(decl_id);
            dep_sym(decl.name.clone())
        }
        Declaration::ImplSelfOrTrait(decl_id) => {
            let decl = engines.pe().get_impl_self_or_trait(decl_id);
            let method_names = decl.items.iter().enumerate().fold(
                String::with_capacity(1024),
                |mut s, (idx, item)| {
                    if idx > 0 {
                        s.push(',');
                    }
                    match item {
                        ImplItem::Fn(id) => engines.pe().map(id, |x| s.push_str(x.name.as_str())),
                        ImplItem::Constant(id) => {
                            engines.pe().map(id, |x| s.push_str(x.name.as_str()))
                        }
                        ImplItem::Type(id) => engines.pe().map(id, |x| s.push_str(x.name.as_str())),
                    }
                    s
                },
            );
            if decl.is_self {
                let trait_name =
                    Ident::new_with_override("self".into(), decl.implementing_for.span());
                impl_sym(
                    trait_name,
                    &type_engine.get(decl.implementing_for.type_id()),
                    method_names,
                )
            } else if decl.trait_name.prefixes.is_empty() {
                impl_sym(
                    decl.trait_name.suffix.clone(),
                    &type_engine.get(decl.implementing_for.type_id()),
                    method_names,
                )
            } else {
                None
            }
        }
        Declaration::ConstGenericDeclaration(_) => {
            todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
        }

        // These don't have declaration dependencies.
        Declaration::VariableDeclaration(_) => None,
        // Storage cannot be depended upon or exported
        Declaration::StorageDeclaration(_) => None,
    }
}

/// This is intentionally different from `Display` for [TypeInfo]
/// because it is used for keys and values in the tree.
fn type_info_name(type_info: &TypeInfo) -> String {
    match type_info {
        TypeInfo::Never => "never",
        TypeInfo::StringArray(_) | TypeInfo::StringSlice => "str",
        TypeInfo::UnsignedInteger(n) => match n {
            IntegerBits::Eight => "uint8",
            IntegerBits::Sixteen => "uint16",
            IntegerBits::ThirtyTwo => "uint32",
            IntegerBits::SixtyFour => "uint64",
            IntegerBits::V256 => "uint256",
        },
        TypeInfo::Boolean => "bool",
        TypeInfo::Custom {
            qualified_call_path: name,
            ..
        } => name.call_path.suffix.as_str(),
        TypeInfo::Tuple(fields) if fields.is_empty() => "unit",
        TypeInfo::Tuple(..) => "tuple",
        TypeInfo::B256 => "b256",
        TypeInfo::Numeric => "numeric",
        TypeInfo::Contract => "contract",
        TypeInfo::ErrorRecovery(_) => "err_recov",
        TypeInfo::Unknown => "unknown",
        TypeInfo::UnknownGeneric { name, .. } => return format!("generic {name}"),
        TypeInfo::TypeParam(_) => "type param",
        TypeInfo::Placeholder(_) => "_",
        TypeInfo::ContractCaller { abi_name, .. } => {
            return format!("contract caller {abi_name}");
        }
        TypeInfo::UntypedEnum(_) => "untyped enum",
        TypeInfo::UntypedStruct(_) => "untyped struct",
        TypeInfo::Struct { .. } => "struct",
        TypeInfo::Enum { .. } => "enum",
        TypeInfo::Array(..) => "array",
        TypeInfo::RawUntypedPtr => "raw untyped ptr",
        TypeInfo::RawUntypedSlice => "raw untyped slice",
        TypeInfo::Ptr(..) => "__ptr",
        TypeInfo::Slice(..) => "__slice",
        TypeInfo::Alias { .. } => "alias",
        TypeInfo::TraitType { .. } => "trait type",
        TypeInfo::Ref { .. } => "reference type",
    }
    .to_string()
}

/// Checks if any dependant depends on a dependee via a chain of dependencies.
fn recursively_depends_on(
    set: &DependencySet,
    dependee: &DependentSymbol,
    decl_dependencies: &DependencyMap,
) -> bool {
    set.contains(dependee)
        || set.iter().any(|dep| {
            decl_dependencies
                .get(dep)
                .map(|dep| recursively_depends_on(&dep.deps, dependee, decl_dependencies))
                .unwrap_or(false)
        })
}
