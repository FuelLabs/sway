use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use crate::type_system::{TypeArgument, TypeParameter};
use crate::TypeEngine;
use crate::{
    error::*,
    language::{parsed::*, CallPath},
    type_system::AbiName,
    TypeInfo,
};

use sway_error::error::CompileError;
use sway_types::integer_bits::IntegerBits;
use sway_types::Spanned;
use sway_types::{ident::Ident, span::Span};

// -------------------------------------------------------------------------------------------------
/// Take a list of nodes and reorder them so that they may be semantically analysed without any
/// dependencies breaking.

pub(crate) fn order_ast_nodes_by_dependency(
    type_engine: &TypeEngine,
    nodes: Vec<AstNode>,
) -> CompileResult<Vec<AstNode>> {
    let decl_dependencies = DependencyMap::from_iter(
        nodes
            .iter()
            .filter_map(|node| Dependencies::gather_from_decl_node(type_engine, node)),
    );

    // Check here for recursive calls now that we have a nice map of the dependencies to help us.
    let mut errors = find_recursive_decls(&decl_dependencies);
    if !errors.is_empty() {
        // Because we're pulling these errors out of a HashMap they'll probably be in a funny
        // order.  Here we'll sort them by span start.
        errors.sort_by_key(|err| err.span().start());
        err(Vec::new(), errors)
    } else {
        // Reorder the parsed AstNodes based on dependency.  Includes first, then uses, then
        // reordered declarations, then anything else.  To keep the list stable and simple we can
        // use a basic insertion sort.
        ok(
            nodes
                .into_iter()
                .fold(Vec::<AstNode>::new(), |ordered, node| {
                    insert_into_ordered_nodes(&decl_dependencies, ordered, node)
                }),
            Vec::new(),
            Vec::new(),
        )
    }
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
        DependentSymbol::Fn(_, Some(fn_span)) => {
            let mut chain = Vec::new();
            find_recursive_call_chain(decl_dependencies, dep_sym, fn_span, &mut chain)
        }
        DependentSymbol::Symbol(_) => {
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
    if let DependentSymbol::Fn(fn_sym_ident, _) = fn_sym {
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
    if let DependentSymbol::Symbol(sym_ident) = dep_sym {
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

type DependencyMap = HashMap<DependentSymbol, Dependencies>;

fn insert_into_ordered_nodes(
    decl_dependencies: &DependencyMap,
    mut ordered_nodes: Vec<AstNode>,
    node: AstNode,
) -> Vec<AstNode> {
    for idx in 0..ordered_nodes.len() {
        // If we find a node which depends on the new node, insert it in front.
        if depends_on(decl_dependencies, &ordered_nodes[idx], &node) {
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

        // Then declarations, ordered using the dependecies list.
        (AstNodeContent::IncludeStatement(_), AstNodeContent::Declaration(_)) => false,
        (AstNodeContent::UseStatement(_), AstNodeContent::Declaration(_)) => false,
        (AstNodeContent::Declaration(dependant), AstNodeContent::Declaration(dependee)) => {
            match (decl_name(dependant), decl_name(dependee)) {
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
    deps: HashSet<DependentSymbol>,
}

impl Dependencies {
    fn gather_from_decl_node(
        type_engine: &TypeEngine,
        node: &AstNode,
    ) -> Option<(DependentSymbol, Dependencies)> {
        match &node.content {
            AstNodeContent::Declaration(decl) => decl_name(decl).map(|name| {
                (
                    name,
                    Dependencies {
                        deps: HashSet::new(),
                    }
                    .gather_from_decl(type_engine, decl),
                )
            }),
            _ => None,
        }
    }

    fn gather_from_decl(self, type_engine: &TypeEngine, decl: &Declaration) -> Self {
        match decl {
            Declaration::VariableDeclaration(VariableDeclaration {
                type_ascription,
                body,
                ..
            }) => self
                .gather_from_typeinfo(type_engine, &type_engine.get(type_ascription.type_id))
                .gather_from_expr(type_engine, body),
            Declaration::ConstantDeclaration(ConstantDeclaration {
                type_ascription,
                value,
                ..
            }) => self
                .gather_from_typeinfo(type_engine, &type_engine.get(type_ascription.type_id))
                .gather_from_expr(type_engine, value),
            Declaration::FunctionDeclaration(fn_decl) => {
                self.gather_from_fn_decl(type_engine, fn_decl)
            }
            Declaration::StructDeclaration(StructDeclaration {
                fields,
                type_parameters,
                ..
            }) => self
                .gather_from_iter(fields.iter(), |deps, field| {
                    deps.gather_from_typeinfo(
                        type_engine,
                        &type_engine.get(field.type_argument.type_id),
                    )
                })
                .gather_from_type_parameters(type_parameters),
            Declaration::EnumDeclaration(EnumDeclaration {
                variants,
                type_parameters,
                ..
            }) => self
                .gather_from_iter(variants.iter(), |deps, variant| {
                    deps.gather_from_typeinfo(
                        type_engine,
                        &type_engine.get(variant.type_argument.type_id),
                    )
                })
                .gather_from_type_parameters(type_parameters),
            Declaration::TraitDeclaration(TraitDeclaration {
                interface_surface,
                methods,
                supertraits,
                ..
            }) => self
                .gather_from_iter(supertraits.iter(), |deps, sup| {
                    deps.gather_from_call_path(&sup.name, false, false)
                })
                .gather_from_iter(interface_surface.iter(), |deps, sig| {
                    deps.gather_from_iter(sig.parameters.iter(), |deps, param| {
                        deps.gather_from_typeinfo(
                            type_engine,
                            &type_engine.get(param.type_argument.type_id),
                        )
                    })
                    .gather_from_typeinfo(type_engine, &sig.return_type)
                })
                .gather_from_iter(methods.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(type_engine, fn_decl)
                }),
            Declaration::ImplTrait(ImplTrait {
                impl_type_parameters,
                trait_name,
                type_implementing_for,
                functions,
                ..
            }) => self
                .gather_from_call_path(trait_name, false, false)
                .gather_from_typeinfo(type_engine, type_implementing_for)
                .gather_from_type_parameters(impl_type_parameters)
                .gather_from_iter(functions.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(type_engine, fn_decl)
                }),
            Declaration::ImplSelf(ImplSelf {
                type_implementing_for,
                functions,
                ..
            }) => self
                .gather_from_typeinfo(type_engine, type_implementing_for)
                .gather_from_iter(functions.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(type_engine, fn_decl)
                }),
            Declaration::AbiDeclaration(AbiDeclaration {
                interface_surface,
                methods,
                ..
            }) => self
                .gather_from_iter(interface_surface.iter(), |deps, sig| {
                    deps.gather_from_iter(sig.parameters.iter(), |deps, param| {
                        deps.gather_from_typeinfo(
                            type_engine,
                            &type_engine.get(param.type_argument.type_id),
                        )
                    })
                    .gather_from_typeinfo(type_engine, &sig.return_type)
                })
                .gather_from_iter(methods.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(type_engine, fn_decl)
                }),
            Declaration::StorageDeclaration(StorageDeclaration { fields, .. }) => self
                .gather_from_iter(
                    fields.iter(),
                    |deps,
                     StorageField {
                         ref type_argument, ..
                     }| {
                        deps.gather_from_typeinfo(
                            type_engine,
                            &type_engine.get(type_argument.type_id),
                        )
                    },
                ),
        }
    }

    fn gather_from_fn_decl(self, type_engine: &TypeEngine, fn_decl: &FunctionDeclaration) -> Self {
        let FunctionDeclaration {
            parameters,
            return_type,
            body,
            type_parameters,
            ..
        } = fn_decl;
        self.gather_from_iter(parameters.iter(), |deps, param| {
            deps.gather_from_typeinfo(type_engine, &type_engine.get(param.type_argument.type_id))
        })
        .gather_from_typeinfo(type_engine, &type_engine.get(return_type.type_id))
        .gather_from_block(type_engine, body)
        .gather_from_type_parameters(type_parameters)
    }

    fn gather_from_expr(self, type_engine: &TypeEngine, expr: &Expression) -> Self {
        match &expr.kind {
            ExpressionKind::Variable(name) => {
                // in the case of ABI variables, we actually want to check if the ABI needs to be
                // ordered
                self.gather_from_call_path(&(name.clone()).into(), false, false)
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                let FunctionApplicationExpression {
                    call_path_binding,
                    arguments,
                } = &**function_application_expression;
                self.gather_from_call_path(&call_path_binding.inner, false, true)
                    .gather_from_type_arguments(
                        type_engine,
                        &call_path_binding.type_arguments.to_vec(),
                    )
                    .gather_from_iter(arguments.iter(), |deps, arg| {
                        deps.gather_from_expr(type_engine, arg)
                    })
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => self
                .gather_from_expr(type_engine, lhs)
                .gather_from_expr(type_engine, rhs),
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
                ..
            }) => if let Some(else_expr) = r#else {
                self.gather_from_expr(type_engine, else_expr)
            } else {
                self
            }
            .gather_from_expr(type_engine, condition)
            .gather_from_expr(type_engine, then),
            ExpressionKind::Match(MatchExpression {
                value, branches, ..
            }) => self
                .gather_from_expr(type_engine, value)
                .gather_from_iter(branches.iter(), |deps, branch| {
                    deps.gather_from_match_branch(type_engine, branch)
                }),
            ExpressionKind::CodeBlock(contents) => self.gather_from_block(type_engine, contents),
            ExpressionKind::Array(array_expression) => self
                .gather_from_iter(array_expression.contents.iter(), |deps, expr| {
                    deps.gather_from_expr(type_engine, expr)
                }),
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index, .. }) => self
                .gather_from_expr(type_engine, prefix)
                .gather_from_expr(type_engine, index),
            ExpressionKind::Struct(struct_expression) => {
                let StructExpression {
                    call_path_binding,
                    fields,
                } = &**struct_expression;
                self.gather_from_call_path(&call_path_binding.inner, false, false)
                    .gather_from_type_arguments(
                        type_engine,
                        &call_path_binding.type_arguments.to_vec(),
                    )
                    .gather_from_iter(fields.iter(), |deps, field| {
                        deps.gather_from_expr(type_engine, &field.value)
                    })
            }
            ExpressionKind::Subfield(SubfieldExpression { prefix, .. }) => {
                self.gather_from_expr(type_engine, prefix)
            }
            ExpressionKind::AmbiguousPathExpression(e) => {
                let AmbiguousPathExpression {
                    call_path_binding,
                    args,
                } = &**e;
                let mut this = self;
                if call_path_binding.inner.prefixes.is_empty() {
                    // We have just `Foo::Bar`, and nothing before `Foo`,
                    // so this could be referring to `Enum::Variant`,
                    // so we want to depend on `Enum` but not `Variant`.
                    this.deps.insert(DependentSymbol::Symbol(
                        call_path_binding.inner.suffix.before.inner.clone(),
                    ));
                }
                this.gather_from_type_arguments(
                    type_engine,
                    &call_path_binding.type_arguments.to_vec(),
                )
                .gather_from_iter(args.iter(), |deps, arg| {
                    deps.gather_from_expr(type_engine, arg)
                })
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
                self.gather_from_call_path(&call_path_binding.inner, true, false)
                    .gather_from_type_arguments(
                        type_engine,
                        &call_path_binding.type_arguments.to_vec(),
                    )
                    .gather_from_iter(args_vec.iter(), |deps, arg| {
                        deps.gather_from_expr(type_engine, arg)
                    })
            }
            ExpressionKind::MethodApplication(method_application_expression) => self
                .gather_from_iter(
                    method_application_expression.arguments.iter(),
                    |deps, arg| deps.gather_from_expr(type_engine, arg),
                ),
            ExpressionKind::Asm(asm) => self
                .gather_from_iter(asm.registers.iter(), |deps, register| {
                    deps.gather_from_opt_expr(type_engine, register.initializer.as_ref())
                })
                .gather_from_typeinfo(type_engine, &asm.return_type),

            // we should do address someday, but due to the whole `re_parse_expression` thing
            // it isn't possible right now
            ExpressionKind::AbiCast(abi_cast_expression) => {
                self.gather_from_call_path(&abi_cast_expression.abi_name, false, false)
            }

            ExpressionKind::Literal(_)
            | ExpressionKind::Break
            | ExpressionKind::Continue
            | ExpressionKind::StorageAccess(_)
            | ExpressionKind::Error(_) => self,

            ExpressionKind::Tuple(fields) => self.gather_from_iter(fields.iter(), |deps, field| {
                deps.gather_from_expr(type_engine, field)
            }),
            ExpressionKind::TupleIndex(TupleIndexExpression { prefix, .. }) => {
                self.gather_from_expr(type_engine, prefix)
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                arguments, ..
            }) => self.gather_from_iter(arguments.iter(), |deps, arg| {
                deps.gather_from_expr(type_engine, arg)
            }),
            ExpressionKind::WhileLoop(WhileLoopExpression {
                condition, body, ..
            }) => self
                .gather_from_expr(type_engine, condition)
                .gather_from_block(type_engine, body),
            ExpressionKind::Reassignment(reassignment) => {
                self.gather_from_expr(type_engine, &reassignment.rhs)
            }
            ExpressionKind::Return(expr) => self.gather_from_expr(type_engine, expr),
        }
    }

    fn gather_from_match_branch(self, type_engine: &TypeEngine, branch: &MatchBranch) -> Self {
        let MatchBranch {
            scrutinee, result, ..
        } = branch;
        self.gather_from_iter(
            scrutinee.gather_approximate_typeinfo_dependencies().iter(),
            |deps, type_info| deps.gather_from_typeinfo(type_engine, type_info),
        )
        .gather_from_expr(type_engine, result)
    }

    fn gather_from_opt_expr(self, type_engine: &TypeEngine, opt_expr: Option<&Expression>) -> Self {
        match opt_expr {
            None => self,
            Some(expr) => self.gather_from_expr(type_engine, expr),
        }
    }

    fn gather_from_block(self, type_engine: &TypeEngine, block: &CodeBlock) -> Self {
        self.gather_from_iter(block.contents.iter(), |deps, node| {
            deps.gather_from_node(type_engine, node)
        })
    }

    fn gather_from_node(self, type_engine: &TypeEngine, node: &AstNode) -> Self {
        match &node.content {
            AstNodeContent::Expression(expr) => self.gather_from_expr(type_engine, expr),
            AstNodeContent::ImplicitReturnExpression(expr) => {
                self.gather_from_expr(type_engine, expr)
            }
            AstNodeContent::Declaration(decl) => self.gather_from_decl(type_engine, decl),

            // No deps from these guys.
            AstNodeContent::UseStatement(_) => self,
            AstNodeContent::IncludeStatement(_) => self,
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
                DependentSymbol::Fn(call_path.suffix.clone(), None)
            } else {
                DependentSymbol::Symbol(call_path.suffix.clone())
            });
        } else if use_prefix && call_path.prefixes.len() == 1 {
            // Here we can use the prefix (e.g., for 'Enum::Variant' -> 'Enum') as long is it's
            // only a single element.
            self.deps
                .insert(DependentSymbol::Symbol(call_path.prefixes[0].clone()));
        }
        self
    }

    fn gather_from_type_parameters(self, type_parameters: &[TypeParameter]) -> Self {
        self.gather_from_iter(type_parameters.iter(), |deps, type_parameter| {
            deps.gather_from_iter(
                type_parameter.trait_constraints.iter(),
                |deps, constraint| deps.gather_from_call_path(&constraint.trait_name, false, false),
            )
        })
    }

    fn gather_from_type_arguments(
        self,
        type_engine: &TypeEngine,
        type_arguments: &[TypeArgument],
    ) -> Self {
        self.gather_from_iter(type_arguments.iter(), |deps, type_argument| {
            deps.gather_from_typeinfo(type_engine, &type_engine.get(type_argument.type_id))
        })
    }

    fn gather_from_typeinfo(mut self, type_engine: &TypeEngine, type_info: &TypeInfo) -> Self {
        match type_info {
            TypeInfo::ContractCaller {
                abi_name: AbiName::Known(abi_name),
                ..
            } => self.gather_from_call_path(abi_name, false, false),
            TypeInfo::Custom {
                call_path: name,
                type_arguments,
            } => {
                self.deps
                    .insert(DependentSymbol::Symbol(name.clone().suffix));
                match type_arguments {
                    Some(type_arguments) => {
                        self.gather_from_type_arguments(type_engine, type_arguments)
                    }
                    None => self,
                }
            }
            TypeInfo::Tuple(elems) => self.gather_from_iter(elems.iter(), |deps, elem| {
                deps.gather_from_typeinfo(type_engine, &type_engine.get(elem.type_id))
            }),
            TypeInfo::Array(elem_type, _) => {
                self.gather_from_typeinfo(type_engine, &type_engine.get(elem_type.type_id))
            }
            TypeInfo::Struct { fields, .. } => {
                self.gather_from_iter(fields.iter(), |deps, field| {
                    deps.gather_from_typeinfo(
                        type_engine,
                        &type_engine.get(field.type_argument.type_id),
                    )
                })
            }
            TypeInfo::Enum { variant_types, .. } => {
                self.gather_from_iter(variant_types.iter(), |deps, variant| {
                    deps.gather_from_typeinfo(
                        type_engine,
                        &type_engine.get(variant.type_argument.type_id),
                    )
                })
            }
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
    Symbol(Ident),
    Fn(Ident, Option<Span>),
    Impl(Ident, String, String), // Trait or self, type implementing for, and method names concatenated.
}

// We'll use a custom Hash and PartialEq here to explicitly ignore the span in the Fn variant.

impl PartialEq for DependentSymbol {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (DependentSymbol::Symbol(l), DependentSymbol::Symbol(r)) => l.eq(r),
            (DependentSymbol::Fn(l, _), DependentSymbol::Fn(r, _)) => l.eq(r),
            (DependentSymbol::Impl(lt, ls, lm), DependentSymbol::Impl(rt, rs, rm)) => {
                lt.eq(rt) && ls.eq(rs) && lm.eq(rm)
            }
            _ => false,
        }
    }
}

use std::hash::{Hash, Hasher};

impl Hash for DependentSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DependentSymbol::Symbol(s) => s.hash(state),
            DependentSymbol::Fn(s, _) => s.hash(state),
            DependentSymbol::Impl(t, s, m) => {
                t.hash(state);
                s.hash(state);
                m.hash(state)
            }
        }
    }
}

fn decl_name(decl: &Declaration) -> Option<DependentSymbol> {
    let dep_sym = |name| Some(DependentSymbol::Symbol(name));
    // `method_names` is the concatenation of all the method names defined in an impl block.
    // This is needed because there can exist multiple impl self blocks for a single type in a
    // file and we need some way to disambiguate them.
    let impl_sym = |trait_name, type_info: &TypeInfo, method_names| {
        Some(DependentSymbol::Impl(
            trait_name,
            type_info_name(type_info),
            method_names,
        ))
    };

    match decl {
        // These declarations can depend upon other declarations.
        Declaration::FunctionDeclaration(decl) => Some(DependentSymbol::Fn(
            decl.name.clone(),
            Some(decl.span.clone()),
        )),
        Declaration::ConstantDeclaration(decl) => dep_sym(decl.name.clone()),
        Declaration::StructDeclaration(decl) => dep_sym(decl.name.clone()),
        Declaration::EnumDeclaration(decl) => dep_sym(decl.name.clone()),
        Declaration::TraitDeclaration(decl) => dep_sym(decl.name.clone()),
        Declaration::AbiDeclaration(decl) => dep_sym(decl.name.clone()),

        // These have the added complexity of converting CallPath and/or TypeInfo into a name.
        Declaration::ImplSelf(decl) => {
            let trait_name =
                Ident::new_with_override("self", decl.type_implementing_for_span.clone());
            impl_sym(
                trait_name,
                &decl.type_implementing_for,
                decl.functions
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(""),
            )
        }
        Declaration::ImplTrait(decl) => {
            if decl.trait_name.prefixes.is_empty() {
                impl_sym(
                    decl.trait_name.suffix.clone(),
                    &decl.type_implementing_for,
                    decl.functions
                        .iter()
                        .map(|x| x.name.as_str())
                        .collect::<Vec<&str>>()
                        .join(""),
                )
            } else {
                None
            }
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
        TypeInfo::Str(_) => "str",
        TypeInfo::UnsignedInteger(n) => match n {
            IntegerBits::Eight => "uint8",
            IntegerBits::Sixteen => "uint16",
            IntegerBits::ThirtyTwo => "uint32",
            IntegerBits::SixtyFour => "uint64",
        },
        TypeInfo::Boolean => "bool",
        TypeInfo::Custom {
            call_path: name, ..
        } => name.suffix.as_str(),
        TypeInfo::Tuple(fields) if fields.is_empty() => "unit",
        TypeInfo::Tuple(..) => "tuple",
        TypeInfo::SelfType => "self",
        TypeInfo::B256 => "b256",
        TypeInfo::Numeric => "numeric",
        TypeInfo::Contract => "contract",
        TypeInfo::ErrorRecovery => "err_recov",
        TypeInfo::Unknown => "unknown",
        TypeInfo::UnknownGeneric { name, .. } => return format!("generic {name}"),
        TypeInfo::Placeholder(_) => "_",
        TypeInfo::ContractCaller { abi_name, .. } => {
            return format!("contract caller {abi_name}");
        }
        TypeInfo::Struct { .. } => "struct",
        TypeInfo::Enum { .. } => "enum",
        TypeInfo::Array(..) => "array",
        TypeInfo::Storage { .. } => "contract storage",
        TypeInfo::RawUntypedPtr => "raw untyped ptr",
        TypeInfo::RawUntypedSlice => "raw untyped slice",
    }
    .to_string()
}

/// Checks if any dependant depends on a dependee via a chain of dependencies.
fn recursively_depends_on(
    set: &HashSet<DependentSymbol>,
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

// -------------------------------------------------------------------------------------------------
//
//
