use indexmap::IndexMap;

use crate::{
    language::{
        ty::{self, TyConstantDecl},
        CallPath, Literal,
    },
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_enum_unsafe_downcast, instantiate_struct_field_access,
            instantiate_tuple_index_access,
        },
        TypeCheckContext,
    },
    Ident, TypeId, UnifyCheck,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};

use sway_types::{span::Span, Named, Spanned};

/// A single requirement in the form `<lhs> == <rhs>` that has to be
/// fulfilled for the match arm to match.
pub(super) type MatchReq = (ty::TyExpression, ty::TyExpression);

/// A single variable in the form `let <ident> = <expression>`
/// that has to be extracted from the match arm.
pub(super) type MatchVarDecl = (Ident, ty::TyExpression);

/// A leaf of a match pattern can be either a requirement on the scrutinee or a
/// variable declaration but not both at the same time.
/// In the case of the catch-all `_` we will have neither a requirement nor
/// a variable declaration.
pub(super) enum ReqOrVarDecl {
    /// Neither a requirement, nor a variable declaration.
    /// Means a catch-all pattern.
    Neither,
    Req(MatchReq),
    VarDecl(MatchVarDecl),
}

/// A tree structure that describes:
/// - the overall requirement that needs to be satisfied in order for the match arm to match
/// - all variable declarations within the match arm
///
/// The tree represents a logical expression that consists of equality comparisons, and
/// lazy AND and OR operators.
///
/// The leaves of the tree are either equality comparisons or eventual variable declarations
/// or none of those in the case of catch-all `_` pattern or only a single rest `..` in structs.
pub(super) struct ReqDeclTree {
    root: ReqDeclNode,
}

impl ReqDeclTree {
    /// Creates a new tree that contains only one leaf node with the
    /// [MatchReq] of the form `<lhs> == <rhs>`.
    fn req(req: MatchReq) -> Self {
        Self {
            root: ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::Req(req)),
        }
    }

    /// Creates a new tree that contains only the leaf node with the
    /// [MatchVarDecl] `decl`.
    fn decl(decl: MatchVarDecl) -> Self {
        Self {
            root: ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::VarDecl(decl)),
        }
    }

    /// Creates a new tree that contains only the leaf node with
    /// neither a requirement nor a variable declaration.
    fn none() -> Self {
        Self {
            root: ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::Neither),
        }
    }

    /// Creates a new tree that contains only an AND node
    /// made of `nodes`.
    fn and(nodes: Vec<ReqDeclNode>) -> Self {
        Self {
            root: ReqDeclNode::And(nodes),
        }
    }

    /// Creates a new tree that contains only an OR node
    /// made of `nodes`.
    fn or(nodes: Vec<ReqDeclNode>) -> Self {
        Self {
            root: ReqDeclNode::Or(nodes),
        }
    }

    pub fn root(&self) -> &ReqDeclNode {
        &self.root
    }
}

/// A single node in the [ReqDeclTree].
pub(super) enum ReqDeclNode {
    /// The leaf node. Contains the information about a single requirement or
    /// variable declaration.
    /// E.g., a catch all `_` will have neither a requirement nor a variable declaration.
    /// E.g., a match arm variable `x` cannot have a requirement (it acts as catch all)
    /// but it will have the declaration of the variable `x`.
    /// E.g., a literal `123` will have a requirement on the scrutinee e.g. `struct.x == 123`.
    ReqOrVarDecl(ReqOrVarDecl),
    /// Represent the requirements and declarations connected with the lazy AND operator,
    /// if there are more then two of them.
    /// Notice that the vector of contained nodes can be empty or have only one element.
    /// AND semantics is applied if there are two or more elements.
    /// E.g., requirements coming from the struct and tuple patterns
    /// must all be fulfilled in order for the whole pattern to match.
    And(Vec<ReqDeclNode>),
    /// Represent the requirements and declarations connected with the lazy OR operator,
    /// if there are more then two of them.
    /// Notice that the vector of contained nodes can be empty or have only one element.
    /// OR semantics is applied if there are two or more elements.
    /// Only the requirements coming from the individual variants of an OR match arm
    /// will be connected with the OR operator.
    Or(Vec<ReqDeclNode>),
}

impl ReqDeclNode {
    /// Creates a new leaf node with the [MatchReq] of the form `<lhs> == <rhs>`.
    fn req(req: MatchReq) -> Self {
        ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::Req(req))
    }

    /// Creates a new leaf node with the [MatchVarDecl] `decl`.
    fn decl(decl: MatchVarDecl) -> Self {
        ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::VarDecl(decl))
    }
}

/// The [matcher] returns the [ReqDeclTree] for the given `scrutinee` that tries
/// to match the given expression `exp`.
///
/// Given the following example:
///
/// ```ignore
/// struct Point {
///     x: u64,
///     y: u64
/// }
///
/// let p = Point {
///     x: 42,
///     y: 24
/// };
///
/// match p {
///     Point { x, y: 5 } => { x },                        // 1.
///     Point { x, y: 5 } | Point { x, y: 10 } => { x },   // 2.
///     Point { x: 10, y: 24 } => { 1 },                   // 3.
///     Point { x: 22, .. } => { 2 },                      // 4.
///     Point { .. } => { 3 },                             // 5.
///     _ => 0                                             // 6.
/// }
/// ```
///
/// the returned [ReqDeclTree] for each match arm will have the following form
/// (square brackets represent each individual leaf node [ReqDeclNode::ReqOrVarDecl]):
///
/// ```ignore
/// 1.
///               &&
///              /  \
/// [let x = p.x]    [p.y == 5]
///
/// 2.
///                             ||
///                 ___________/  \____________
///               &&                           &&
///              /  \                         /  \
/// [let x = p.x]    [p.y == 5]  [let x = p.x]    [p.y == 10]
///
/// 3.
///             &&
///            /  \
/// [p.x == 10]    [p.y == 24]
///
/// 4.
///      &&      // Note that this AND node has only one childe node.
///      |
/// [p.x == 22]
///
/// 5.
///   &&      // Note that this AND node has only one childe node.
///   |
/// [None]
///
/// 6.
/// [None]
/// ```
pub(super) fn matcher(
    handler: &Handler,
    ctx: TypeCheckContext,
    match_value: &ty::TyExpression,
    exp: &ty::TyExpression,
    scrutinee: ty::TyScrutinee,
) -> Result<ReqDeclTree, ErrorEmitted> {
    let ty::TyScrutinee {
        variant,
        type_id,
        span,
    } = scrutinee;

    let type_engine = ctx.engines.te();

    // unify the type of the scrutinee with the type of the expression
    handler.scope(|h| {
        type_engine.unify(h, ctx.engines, type_id, exp.return_type, &span, "", None);
        Ok(())
    })?;

    match variant {
        ty::TyScrutineeVariant::Or(alternatives) => {
            match_or(handler, ctx, match_value, exp, alternatives)
        }
        ty::TyScrutineeVariant::CatchAll => Ok(ReqDeclTree::none()),
        ty::TyScrutineeVariant::Literal(value) => Ok(match_literal(exp, value, span)),
        ty::TyScrutineeVariant::Variable(name) => Ok(match_variable(exp, name)),
        ty::TyScrutineeVariant::Constant(_, _, const_decl) => {
            Ok(match_constant(ctx, exp, const_decl, span))
        }
        ty::TyScrutineeVariant::StructScrutinee {
            struct_ref: _,
            fields,
            ..
        } => match_struct(handler, ctx, match_value, exp, fields),
        ty::TyScrutineeVariant::EnumScrutinee {
            variant,
            call_path_decl,
            value,
            ..
        } => match_enum(
            handler,
            ctx,
            match_value,
            exp,
            *variant,
            call_path_decl,
            *value,
            span,
        ),
        ty::TyScrutineeVariant::Tuple(elems) => {
            match_tuple(handler, ctx, match_value, exp, elems, span)
        }
    }
}

fn match_or(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    match_value: &ty::TyExpression,
    exp: &ty::TyExpression,
    alternatives: Vec<ty::TyScrutinee>,
) -> Result<ReqDeclTree, ErrorEmitted> {
    return handler.scope(|handler| {
        let mut nodes = vec![];
        let mut variables_in_alternatives: Vec<(Span, Vec<(Ident, TypeId)>)> = vec![]; // Span is the span of the alternative.

        for alternative in alternatives {
            let alternative_span = alternative.span.clone();

            // We want to collect as many errors as possible.
            // If an alternative has any internal issues we will emit them, ignore that alternative,
            // but still process the remaining alternatives.
            let alternative_req_decl_tree =
                match matcher(handler, ctx.by_ref(), match_value, exp, alternative) {
                    Ok(req_decl_tree) => req_decl_tree,
                    Err(_) => continue,
                };

            variables_in_alternatives.push((
                alternative_span,
                variable_declarations(&alternative_req_decl_tree),
            ));

            nodes.push(alternative_req_decl_tree.root);
        }

        // All the first occurrences of variables in order of appearance.
        let mut variables: IndexMap<&Ident, TypeId> = IndexMap::new();
        for (ident, type_id) in variables_in_alternatives.iter().flat_map(|(_, vars)| vars) {
            variables.entry(ident).or_insert(*type_id);
        }

        // At this stage, in the matcher, we are not concerned about the duplicates
        // in individual alternatives.

        // Check that we have all variables in all alternatives.
        for (variable, _) in variables.iter() {
            let missing_in_alternatives: Vec<Span> = variables_in_alternatives
                .iter()
                .filter_map(|(span, vars)| {
                    (!vars.iter().any(|(ident, _)| ident == *variable)).then_some(span.clone())
                })
                .collect();

            if missing_in_alternatives.is_empty() {
                continue;
            }

            handler.emit_err(CompileError::MatchArmVariableNotDefinedInAllAlternatives {
                match_value: match_value.span.clone(),
                match_type: ctx.engines.help_out(match_value.return_type).to_string(),
                variable: (*variable).clone(),
                missing_in_alternatives,
            });
        }

        // Check that the variable types are the same in all alternatives
        // (assuming that the variable exist in the alternative).

        // To the equality, we accept type aliases and the types they encapsulate
        // to be equal, otherwise, we are strict, e.g., no coercion between u8 and u16, etc.
        let equality = UnifyCheck::non_dynamic_equality(ctx.engines);

        for (variable, type_id) in variables {
            let type_mismatched_vars = variables_in_alternatives.iter().flat_map(|(_, vars)| {
                vars.iter().filter_map(|(ident, var_type_id)| {
                    (ident == variable && !equality.check(type_id, *var_type_id))
                        .then_some((ident.clone(), *var_type_id))
                })
            });

            for type_mismatched_var in type_mismatched_vars {
                handler.emit_err(CompileError::MatchArmVariableMismatchedType {
                    match_value: match_value.span.clone(),
                    match_type: ctx.engines.help_out(match_value.return_type).to_string(),
                    variable: type_mismatched_var.0,
                    first_definition: variable.span(),
                    expected: ctx.engines.help_out(type_id).to_string(),
                    received: ctx.engines.help_out(type_mismatched_var.1).to_string(),
                });
            }
        }

        Ok(ReqDeclTree::or(nodes))
    });

    /// Returns all [MatchVarDecl]s found in the match arm
    /// in order of their appearance from left to right.
    fn variable_declarations(req_decl_tree: &ReqDeclTree) -> Vec<(Ident, TypeId)> {
        let mut result = vec![];

        collect_variable_declarations(&req_decl_tree.root, &mut result);

        return result;

        fn collect_variable_declarations(
            node: &ReqDeclNode,
            declarations: &mut Vec<(Ident, TypeId)>,
        ) {
            // Traverse the tree depth-first, left to right.
            match node {
                ReqDeclNode::ReqOrVarDecl(ReqOrVarDecl::VarDecl((ident, exp))) => {
                    declarations.push((ident.clone(), exp.return_type));
                }
                ReqDeclNode::ReqOrVarDecl(_) => (),
                ReqDeclNode::And(nodes) | ReqDeclNode::Or(nodes) => {
                    for node in nodes {
                        collect_variable_declarations(node, declarations);
                    }
                }
            }
        }
    }
}

fn match_literal(exp: &ty::TyExpression, scrutinee: Literal, span: Span) -> ReqDeclTree {
    let req = (
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(scrutinee),
            return_type: exp.return_type,
            span,
        },
    );

    ReqDeclTree::req(req)
}

fn match_variable(exp: &ty::TyExpression, scrutinee_name: Ident) -> ReqDeclTree {
    let decl = (scrutinee_name, exp.to_owned());

    ReqDeclTree::decl(decl)
}

fn match_constant(
    ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    const_decl: TyConstantDecl,
    span: Span,
) -> ReqDeclTree {
    let name = const_decl.name().clone();
    let return_type = const_decl.type_ascription.type_id();

    let req = (
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::ConstantExpression {
                span: span.clone(),
                decl: Box::new(const_decl),
                call_path: Some(CallPath::from(name).to_fullpath(ctx.engines(), ctx.namespace())),
            },
            return_type,
            span,
        },
    );

    ReqDeclTree::req(req)
}

fn match_struct(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    match_value: &ty::TyExpression,
    exp: &ty::TyExpression,
    fields: Vec<ty::TyStructScrutineeField>,
) -> Result<ReqDeclTree, ErrorEmitted> {
    let mut nodes = vec![];

    for ty::TyStructScrutineeField {
        field,
        scrutinee,
        span: field_span,
        field_def_name: _,
    } in fields.into_iter()
    {
        // Get the expression that access the struct field e.g., `my_struct.x`.
        let subfield = instantiate_struct_field_access(
            handler,
            ctx.engines(),
            ctx.namespace(),
            exp.clone(),
            field.clone(),
            field_span,
        )?;

        match scrutinee {
            // If there is no scrutinee, we simply have the struct field name.
            // This means declaring a variable with the same name as the struct field,
            // initialized to the values of the subfield expression.
            None => {
                nodes.push(ReqDeclNode::decl((field, subfield)));
            }
            // If the scrutinee exist, we have the form `<field>: <match_sub_pattern>`.
            // We need to match the subfield against the sub pattern.
            Some(match_sub_pattern) => {
                let req_decl_tree = matcher(
                    handler,
                    ctx.by_ref(),
                    match_value,
                    &subfield,
                    match_sub_pattern,
                )?;
                nodes.push(req_decl_tree.root);
            }
        }
    }

    Ok(ReqDeclTree::and(nodes))
}

#[allow(clippy::too_many_arguments)]
fn match_enum(
    handler: &Handler,
    ctx: TypeCheckContext,
    match_value: &ty::TyExpression,
    exp: &ty::TyExpression,
    variant: ty::TyEnumVariant,
    call_path_decl: ty::TyDecl,
    enum_value_scrutinee: ty::TyScrutinee,
    span: Span,
) -> Result<ReqDeclTree, ErrorEmitted> {
    let type_engine = ctx.engines.te();

    let mut nodes = vec![];

    // The first requirement is that the enum variant behind the `exp` is
    // of the kind `variant`. `exp is variant` is expressed as `EnumTag(<exp>) == <variant.tag>`.
    let enum_variant_req = (
        ty::TyExpression {
            expression: ty::TyExpressionVariant::EnumTag {
                exp: Box::new(exp.clone()),
            },
            return_type: type_engine.id_of_u64(),
            span: exp.span.clone(),
        },
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(Literal::U64(variant.tag as u64)),
            return_type: type_engine.id_of_u64(),
            span: exp.span.clone(),
        },
    );

    nodes.push(ReqDeclNode::req(enum_variant_req));

    // Afterwards, we need to collect the requirements for the enum variant underlying value.
    // If the enum variant does not have a value the `enum_value_scrutinee` will be of the
    // scrutinee variant `CatchAll` that will produce a ReqDeclTree without further requirements
    // or variable declarations.
    let unsafe_downcast = instantiate_enum_unsafe_downcast(exp, variant, call_path_decl, span);
    let req_decl_tree = matcher(
        handler,
        ctx,
        match_value,
        &unsafe_downcast,
        enum_value_scrutinee,
    )?;

    nodes.push(req_decl_tree.root);

    Ok(ReqDeclTree::and(nodes))
}

fn match_tuple(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    match_value: &ty::TyExpression,
    exp: &ty::TyExpression,
    elems: Vec<ty::TyScrutinee>,
    span: Span,
) -> Result<ReqDeclTree, ErrorEmitted> {
    let mut nodes = vec![];

    for (pos, elem) in elems.into_iter().enumerate() {
        let tuple_index_access = instantiate_tuple_index_access(
            handler,
            ctx.engines(),
            exp.clone(),
            pos,
            span.clone(),
            span.clone(),
        )?;

        let req_decl_tree = matcher(
            handler,
            ctx.by_ref(),
            match_value,
            &tuple_index_access,
            elem,
        )?;

        nodes.push(req_decl_tree.root);
    }

    Ok(ReqDeclTree::and(nodes))
}
