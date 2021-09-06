use crate::{
    parse_tree::*, types::IntegerBits, AstNode, AstNodeContent, CodeBlock, Declaration, Expression,
    ReturnStatement, TypeInfo, WhileLoop,
};

/// Take a list of nodes and reorder them so that they may be semantically analysed without any
/// dependencies breaking.

pub(crate) fn order_ast_nodes_by_dependency<'sc>(nodes: &[AstNode<'sc>]) -> Vec<AstNode<'sc>> {
    let decl_dependencies: Vec<(DependentSymbol, Dependencies)> = nodes
        .iter()
        .filter_map(|node| Dependencies::gather_from_decl_node(node))
        .collect();

    // Reorder the parsed AstNodes based on dependency.  Includes first, then uses, then
    // reordered declarations, then anything else.  To keep the list stable and simple we can
    // use a basic insertion sort.
    nodes
        .iter()
        .fold(Vec::<AstNode<'sc>>::new(), |ordered, node| {
            insert_into_ordered_nodes(&decl_dependencies, ordered, node.clone())
        })
}

fn insert_into_ordered_nodes<'sc>(
    decl_dependencies: &[(DependentSymbol, Dependencies)],
    mut ordered_nodes: Vec<AstNode<'sc>>,
    node: AstNode<'sc>,
) -> Vec<AstNode<'sc>> {
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

fn depends_on<'sc>(
    decl_dependencies: &[(DependentSymbol, Dependencies)],
    dependant_node: &AstNode<'sc>,
    dependee_node: &AstNode<'sc>,
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
                (Some(dependant_name), Some(dependee_name)) => {
                    // Rather than iterating through each dependency we could use a HashMap here.
                    decl_dependencies.iter().any(|(key, names)| {
                        // Is this the right list of dependencies and is the dependent in the list?
                        key == &dependant_name
                            && names.deps.iter().any(|name| dependee_name.is(name))
                    })
                }
                _ => false,
            }
        }
        (_, AstNodeContent::Declaration(_)) => true,

        // Everything else we don't care.
        _ => false,
    }
}

// -------------------------------------------------------------------------------------------------

struct Dependencies<'sc> {
    deps: Vec<&'sc str>,
}

impl<'sc> Dependencies<'sc> {
    fn gather_from_decl_node(
        node: &'sc AstNode<'sc>,
    ) -> Option<(DependentSymbol<'sc>, Dependencies<'sc>)> {
        match &node.content {
            AstNodeContent::Declaration(decl) => decl_name(decl).map(|name| {
                (
                    name,
                    Dependencies { deps: Vec::new() }.gather_from_decl(decl),
                )
            }),
            _ => None,
        }
    }

    fn gather_from_decl(self, decl: &'sc Declaration) -> Self {
        match decl {
            Declaration::VariableDeclaration(VariableDeclaration {
                type_ascription,
                body,
                ..
            }) => self
                .gather_from_option_typeinfo(&type_ascription)
                .gather_from_expr(&body),
            Declaration::FunctionDeclaration(fn_decl) => self.gather_from_fn_decl(fn_decl),
            Declaration::StructDeclaration(StructDeclaration {
                fields,
                type_parameters,
                ..
            }) => self
                .gather_from_iter(fields.iter(), |deps, field| {
                    deps.gather_from_typeinfo(&field.r#type)
                })
                .gather_from_traits(type_parameters),
            Declaration::EnumDeclaration(EnumDeclaration {
                variants,
                type_parameters,
                ..
            }) => self
                .gather_from_iter(variants.iter(), |deps, variant| {
                    deps.gather_from_typeinfo(&variant.r#type)
                })
                .gather_from_traits(type_parameters),
            Declaration::Reassignment(decl) => self.gather_from_expr(&decl.rhs),
            Declaration::TraitDeclaration(TraitDeclaration {
                interface_surface,
                methods,
                type_parameters,
                ..
            }) => self
                .gather_from_iter(interface_surface.iter(), |deps, sig| {
                    deps.gather_from_iter(sig.parameters.iter(), |deps, param| {
                        deps.gather_from_typeinfo(&param.r#type)
                    })
                    .gather_from_typeinfo(&sig.return_type)
                })
                .gather_from_iter(methods.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(fn_decl)
                })
                .gather_from_traits(type_parameters),
            Declaration::ImplTrait(ImplTrait {
                trait_name,
                type_implementing_for,
                type_arguments,
                functions,
                ..
            }) => self
                .gather_from_call_path(trait_name, false)
                .gather_from_typeinfo(type_implementing_for)
                .gather_from_traits(type_arguments)
                .gather_from_iter(functions.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(fn_decl)
                }),
            Declaration::ImplSelf(ImplSelf {
                type_implementing_for,
                type_arguments,
                functions,
                ..
            }) => self
                .gather_from_typeinfo(type_implementing_for)
                .gather_from_traits(type_arguments)
                .gather_from_iter(functions.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(fn_decl)
                }),
            Declaration::AbiDeclaration(AbiDeclaration {
                interface_surface,
                methods,
                ..
            }) => self
                .gather_from_iter(interface_surface.iter(), |deps, sig| {
                    deps.gather_from_iter(sig.parameters.iter(), |deps, param| {
                        deps.gather_from_typeinfo(&param.r#type)
                    })
                    .gather_from_typeinfo(&sig.return_type)
                })
                .gather_from_iter(methods.iter(), |deps, fn_decl| {
                    deps.gather_from_fn_decl(fn_decl)
                }),
        }
    }

    fn gather_from_fn_decl(self, fn_decl: &'sc FunctionDeclaration<'sc>) -> Self {
        let FunctionDeclaration {
            parameters,
            return_type,
            body,
            type_parameters,
            ..
        } = fn_decl;
        self.gather_from_iter(parameters.iter(), |deps, param| {
            deps.gather_from_typeinfo(&param.r#type)
        })
        .gather_from_typeinfo(&return_type)
        .gather_from_block(&body)
        .gather_from_traits(&type_parameters)
    }

    fn gather_from_expr(mut self, expr: &'sc Expression) -> Self {
        match expr {
            Expression::VariableExpression { .. } => self,
            Expression::FunctionApplication {
                name, arguments, ..
            } => self
                .gather_from_call_path(name, false)
                .gather_from_iter(arguments.iter(), |deps, arg| deps.gather_from_expr(arg)),
            Expression::IfExp {
                condition,
                then,
                r#else,
                ..
            } => if let Some(else_expr) = r#else {
                self.gather_from_expr(else_expr)
            } else {
                self
            }
            .gather_from_expr(condition)
            .gather_from_expr(then),
            Expression::CodeBlock { contents, .. } => self.gather_from_block(contents),
            Expression::Array { contents, .. } => {
                self.gather_from_iter(contents.iter(), |deps, expr| deps.gather_from_expr(expr))
            }
            Expression::StructExpression {
                struct_name,
                fields,
                ..
            } => {
                self.deps.push(struct_name.primary_name);
                self.gather_from_iter(fields.iter(), |deps, field| {
                    deps.gather_from_expr(&field.value)
                })
            }
            Expression::SubfieldExpression { prefix, .. } => self.gather_from_expr(prefix),
            Expression::DelineatedPath { call_path, .. } => {
                // It's either a module path which we can ignore, or an enum variant path, in which
                // case we're interested in the enum name, ignoring the variant name.
                self.gather_from_call_path(call_path, true)
            }
            Expression::MethodApplication { arguments, .. } => {
                self.gather_from_iter(arguments.iter(), |deps, arg| deps.gather_from_expr(arg))
            }
            Expression::AsmExpression { asm, .. } => self
                .gather_from_iter(asm.registers.iter(), |deps, register| {
                    deps.gather_from_opt_expr(&register.initializer)
                })
                .gather_from_typeinfo(&asm.return_type),
            Expression::MatchExpression {
                primary_expression,
                branches,
                ..
            } => self.gather_from_expr(primary_expression).gather_from_iter(
                branches.iter(),
                |deps, branch| {
                    match &branch.condition {
                        MatchCondition::CatchAll => deps,
                        MatchCondition::Expression(expr) => deps.gather_from_expr(expr),
                    }
                    .gather_from_expr(&branch.result)
                },
            ),

            // Not sure about AbiCast, could add the abi_name and address.
            Expression::AbiCast { .. } => self,

            Expression::Literal { .. } => self,
            Expression::Unit { .. } => self,
        }
    }

    fn gather_from_opt_expr(self, opt_expr: &'sc Option<Expression>) -> Self {
        match opt_expr {
            None => self,
            Some(expr) => self.gather_from_expr(expr),
        }
    }

    fn gather_from_block(self, block: &'sc CodeBlock) -> Self {
        self.gather_from_iter(block.contents.iter(), |deps, node| {
            deps.gather_from_node(node)
        })
    }

    fn gather_from_node(self, node: &'sc AstNode<'sc>) -> Self {
        match &node.content {
            AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                self.gather_from_expr(&expr)
            }
            AstNodeContent::Expression(expr) => self.gather_from_expr(&expr),
            AstNodeContent::ImplicitReturnExpression(expr) => self.gather_from_expr(&expr),
            AstNodeContent::Declaration(decl) => self.gather_from_decl(&decl),
            AstNodeContent::WhileLoop(WhileLoop { condition, body }) => {
                self.gather_from_expr(&condition).gather_from_block(&body)
            }

            // No deps from these guys.
            AstNodeContent::UseStatement(_) => self,
            AstNodeContent::IncludeStatement(_) => self,
        }
    }

    fn gather_from_call_path(mut self, call_path: &'sc CallPath, use_prefix: bool) -> Self {
        if call_path.prefixes.is_empty() {
            // We can just use the suffix.
            self.deps.push(call_path.suffix.primary_name);
        } else if use_prefix && call_path.prefixes.len() == 1 {
            // Here we can use the prefix (e.g., for 'Enum::Variant' -> 'Enum') as long is it's
            // only a single element.
            self.deps.push(call_path.prefixes[0].primary_name);
        }
        self
    }

    fn gather_from_traits(mut self, type_parameters: &[TypeParameter<'sc>]) -> Self {
        for type_param in type_parameters {
            for constraint in &type_param.trait_constraints {
                self.deps.push(constraint.name.primary_name)
            }
        }
        self
    }

    fn gather_from_typeinfo(mut self, type_info: &'sc TypeInfo<'sc>) -> Self {
        if let TypeInfo::Custom { name } = type_info {
            self.deps.push(name.primary_name);
        }
        self
    }

    fn gather_from_option_typeinfo(self, opt_type_info: &'sc Option<TypeInfo<'sc>>) -> Self {
        match opt_type_info {
            None => self,
            Some(type_info) => self.gather_from_typeinfo(type_info),
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

// PartialEq is required only for matching the keys of the dependency map.  If we were to use a
// HashMap then we'd need Eq and Hash too.
#[derive(Debug, PartialEq)]
enum DependentSymbol<'sc> {
    Symbol(&'sc str),
    Impl(&'sc str, &'sc str), // Trait or self, and type implementing for.
}

impl<'sc> DependentSymbol<'sc> {
    fn is(&self, sym: &str) -> bool {
        match self {
            DependentSymbol::Symbol(s) => &sym == s,
            DependentSymbol::Impl(..) => false,
        }
    }
}

fn decl_name<'sc>(decl: &'sc Declaration) -> Option<DependentSymbol<'sc>> {
    let dep_sym = |name| Some(DependentSymbol::Symbol(name));
    let impl_sym = |trait_name, type_info: &'sc TypeInfo| {
        Some(DependentSymbol::Impl(trait_name, type_info_name(type_info)))
    };

    match decl {
        // These declarations can depend upon other declarations.
        Declaration::FunctionDeclaration(decl) => dep_sym(decl.name.primary_name),
        Declaration::StructDeclaration(decl) => dep_sym(decl.name.primary_name),
        Declaration::EnumDeclaration(decl) => dep_sym(decl.name.primary_name),
        Declaration::TraitDeclaration(decl) => dep_sym(decl.name.primary_name),
        Declaration::AbiDeclaration(decl) => dep_sym(decl.name.primary_name),

        // These have the added complexity of converting CallPath and/or TypeInfo into a name.
        Declaration::ImplSelf(decl) => impl_sym("self", &decl.type_implementing_for),
        Declaration::ImplTrait(decl) => {
            if decl.trait_name.prefixes.is_empty() {
                impl_sym(
                    decl.trait_name.suffix.primary_name,
                    &decl.type_implementing_for,
                )
            } else {
                None
            }
        }

        // These don't have declaration dependencies.
        Declaration::VariableDeclaration(_) => None,
        Declaration::Reassignment(_) => None,
    }
}

fn type_info_name<'sc>(type_info: &TypeInfo<'sc>) -> &'sc str {
    match type_info {
        TypeInfo::Str(_) => "str",
        TypeInfo::UnsignedInteger(n) => match n {
            IntegerBits::Eight => "uint8",
            IntegerBits::Sixteen => "uint16",
            IntegerBits::ThirtyTwo => "uint32",
            IntegerBits::SixtyFour => "uint64",
        },
        TypeInfo::Boolean => "bool",
        TypeInfo::Custom { name } => name.primary_name,
        TypeInfo::Unit => "unit",
        TypeInfo::SelfType => "self",
        TypeInfo::Byte => "byte",
        TypeInfo::B256 => "b256",
        TypeInfo::Numeric => "numeric",
        TypeInfo::Contract => "contract",
        TypeInfo::ErrorRecovery => "err_recov",
    }
}
