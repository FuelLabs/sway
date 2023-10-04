use either::Either;
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    language::{ty::{self}, CallPath, Literal},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_struct_field_access, instantiate_tuple_index_access,
            instantiate_enum_unsafe_downcast,
        },
        TypeCheckContext, typed_expression::{instantiate_lazy_or, instantiate_lazy_and},
    },
    Ident, TypeId, UnifyCheck, TypeInfo,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};

use sway_types::{span::Span, Spanned, integer_bits::IntegerBits};

/// A single requirement that the desugared if expression must include
/// in its condition. The requirement is in the form `<lhs> == <rhs>`.
#[derive(Debug)] // TODO-IG: Remove.
pub(crate) struct MatchReq { // TODO-IG: Back to tuple.
    lhs: ty::TyExpression,
    rhs: ty::TyExpression,
}

impl MatchReq {
    fn new(lhs: ty::TyExpression, rhs: ty::TyExpression) -> Self {
        Self {
            lhs,
            rhs,
        }
    }

    // TODO-IG: Remove. Matcher should not do desugaring.
    /// Returns a boolean expression of the form `<lhs> == <rhs>`.
    pub(crate) fn to_requirement_expression(&self, handler: &Handler, ctx: TypeCheckContext) -> Result<ty::TyExpression, ErrorEmitted> {
        ty::TyExpression::core_ops_eq(
            handler,
            ctx,
            vec![self.lhs.clone(), self.rhs.clone()],
            Span::join(self.lhs.span.clone(), self.rhs.span.clone()),
        )
    }
}

/// A single variable declaration that the desugared if expression body must include.
/// The declaration is in the form `let <ident> = <expression>`.
pub(crate) type MatchVarDecl = (Ident, ty::TyExpression);

/// A leaf of a match pattern can be either a requirement on the scrutinee or a
/// variable declaration but not both at the same time.
/// In the case of the catch-all `_` we will have neither a requirement nor
/// a variable declaration.
pub(crate) type ReqOrVarDecl = Option<Either<MatchReq, MatchVarDecl>>;

/// A tree structure that describes:
/// - the overall requirement that needs to be satisfied in order for the match arm to match
/// - all variable declarations within the match arm
/// The tree represents a logical expression that consists of equality comparisons, and
/// lazy AND and OR operators.
/// The leafs of the tree are either equality comparisons or eventual variable declarations
/// or none of those in the case of catch-all `_` pattern or only a single rest `..` in structs.
#[derive(Debug)] // TODO-IG: Remove.
pub(crate) struct ReqDeclTree {
    pub root: ReqDeclNode,
    _priv: (), // Only the matcher can create trees of requirements and declarations.
}

impl ReqDeclTree {
    /// Creates a new tree that contains only one leaf node with the
    /// [MatchReq] of the form `<lhs> == <rhs>`.
    fn req(lhs: ty::TyExpression, rhs: ty::TyExpression) -> Self {
        let req = MatchReq::new(lhs, rhs);
        Self {
            root: ReqDeclNode::ReqOrVarDecl(Some(Either::Left(req))),
            _priv: (),
        }
    }

    /// Creates a new tree that contains only the leaf node with the
    /// [MatchVarDecl] `decl`.
    fn decl(decl: MatchVarDecl) -> Self {
        Self {
            root: ReqDeclNode::ReqOrVarDecl(Some(Either::Right(decl))),
            _priv: (),
        }
    }

    /// Creates a new tree that contains only the leaf node with
    /// neither a requirement nor a variable declaration.
    fn none() -> Self {
        Self {
            root: ReqDeclNode::ReqOrVarDecl(None),
            _priv: (),
        }
    }

    /// Creates a new tree that contains only an AND node
    /// made of `nodes`.
    fn and(nodes: Vec<ReqDeclNode>) -> Self {
        Self {
            root: ReqDeclNode::And(nodes),
            _priv: (),
        }
    }

    /// Creates a new tree that contains only an OR node
    /// made of `nodes`.
    fn or(nodes: Vec<ReqDeclNode>) -> Self {
        Self {
            root: ReqDeclNode::Or(nodes),
            _priv: (),
        }
    }

    /// Returns all [MatchVarDecl]s found in the match arm
    /// in order of their appearance from left to right.
    pub(crate) fn variable_declarations(&self) -> Vec<&MatchVarDecl> {
        let mut result = vec![];

        collect_variable_declarations(&self.root, &mut result);

        return result;

        fn collect_variable_declarations<'a>(node: &'a ReqDeclNode, declarations: &mut Vec<&'a MatchVarDecl>) {
            // Traverse the tree depth-first, left to right.
            match node {
                ReqDeclNode::ReqOrVarDecl(Some(Either::Right(decl))) => {
                    declarations.push(decl);
                },
                ReqDeclNode::ReqOrVarDecl(_) => (),
                ReqDeclNode::And(nodes) | ReqDeclNode::Or(nodes) => {
                    for node in nodes {
                        collect_variable_declarations(node, declarations);
                    }
                },
            }
        }
    }

    // TODO-IG: Remove from here. Matcher should not do desugaring! Same with the MatchReq struct -> back to tuple.
    /// Returns a boolean expression that represents the total match arm requirement,
    /// or `None` if the match arm is a catch-all arm.
    /// E.g.: `struct.x == 11 && struct.y == 22 || struct.x == 33 && struct.y == 44`
    /// In case of error, the returned expression will be a structurally valid boolean expression but semantically
    /// invalid, e.a., it will not represent the original semantics of the match arm pattern.
    /// This way we can proceed with the compilation and collect more errors, while still knowing via scoped handlers
    /// that an error happened.
    pub(crate) fn to_requirement_expression(&self, handler: &Handler, ctx: &mut TypeCheckContext) -> Option<ty::TyExpression> {
        return convert_req_decl_node_to_req_exp(handler, ctx, &self.root);

        fn convert_req_decl_node_to_req_exp(handler: &Handler, ctx: &mut TypeCheckContext, req_decl_node: &ReqDeclNode) -> Option<ty::TyExpression> {
            return match req_decl_node {
                ReqDeclNode::ReqOrVarDecl(Some(Either::Left(req))) => req.to_requirement_expression(handler, ctx.by_ref()).ok(),
                ReqDeclNode::ReqOrVarDecl(_) => None,
                ReqDeclNode::And(nodes) => {
                    convert_and_or_or_req_decl_node(handler, ctx, nodes, |lhs, rhs| instantiate_lazy_and(ctx.engines, lhs, rhs))
                },
                ReqDeclNode::Or(nodes) => {
                    convert_and_or_or_req_decl_node(handler, ctx, nodes, |lhs, rhs| instantiate_lazy_or(ctx.engines, lhs, rhs))
                },
            };

            fn convert_and_or_or_req_decl_node(handler: &Handler, ctx: &mut TypeCheckContext, nodes: &Vec<ReqDeclNode>, operator: impl Fn(ty::TyExpression, ty::TyExpression) -> ty::TyExpression) -> Option<ty::TyExpression> {
                // If any of the nodes cannot be converted, the errors will be collected and we proceed with building a semantically invalid boolean expression.
                let req_nodes = &nodes.iter().filter_map(|node| convert_req_decl_node_to_req_exp(handler, ctx, node)).collect_vec()[..];
                match req_nodes {
                    [] => None,
                    _ => Some(build_expression(req_nodes, &operator)),
                }
            }
        
            fn build_expression(expressions: &[ty::TyExpression], operator: &impl Fn(ty::TyExpression, ty::TyExpression) -> ty::TyExpression) -> ty::TyExpression {
                let (lhs, others) = expressions.split_first().expect("The slice of requirement expressions must not be empty.");
                match others {
                    [] => lhs.clone(),
                    _ => operator(lhs.clone(), build_expression(others, operator)),
                }
            }
        }
    }
}

/// A single node in the [ReqDeclTree].
#[derive(Debug)] // TODO-IG: Remove.
pub(crate) enum ReqDeclNode {
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
    fn req(lhs: ty::TyExpression, rhs: ty::TyExpression) -> Self {
        ReqDeclNode::ReqOrVarDecl(Some(Either::Left(MatchReq::new(lhs, rhs))))
    }

    /// Creates a new leaf node with the [MatchVarDecl] `decl`.
    fn decl(decl: MatchVarDecl) -> Self {
        ReqDeclNode::ReqOrVarDecl(Some(Either::Right(decl)))
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
/// (square brackets represent each individual leaf node [ReqDeclNode::ReqVarDecl]):
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
pub(crate) fn matcher(
    handler: &Handler,
    mut ctx: TypeCheckContext,
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
    let engines = ctx.engines();

    // unify the type of the scrutinee with the type of the expression
    handler.scope(|h| {
        type_engine.unify(h, engines, type_id, exp.return_type, &span, "", None);
        Ok(())
    })?;

    match variant {
        ty::TyScrutineeVariant::Or(alternatives) => handler.scope(|handler| { // TODO-IG: Move into separate function.
            let mut nodes = vec![];
            let mut variables_in_alternatives: Vec<(Span, Vec<MatchVarDecl>)> = vec![]; // Span is the span of the alternative.

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

                // TODO-IG: Implement without cloning whole MatchVarDecls.
                variables_in_alternatives.push((alternative_span, alternative_req_decl_tree.variable_declarations().into_iter().map(|decl| decl.clone()).collect()));

                nodes.push(alternative_req_decl_tree);
            }

            // All the first occurrences of variables in order of appearance.
            let mut variables: IndexMap<&Ident, TypeId> = IndexMap::new();
            for (ident, expr) in variables_in_alternatives
                .iter()
                .flat_map(|(_, declarations)| declarations)
            {
                variables.entry(ident).or_insert(expr.return_type);
            }

            // At this stage, in the matcher, we are not concerned about the duplicates
            // in individual alternatives.

            // Check that we have all variables in all alternatives.
            for (variable, _) in variables.iter() {
                let missing_in_alternatives: Vec<Span> = variables_in_alternatives
                    .iter()
                    .filter(|(_, declarations)| !declarations.iter().any(|(ident, _)| ident == *variable))
                    .map(|(span, _)| span.clone())
                    .collect();

                if missing_in_alternatives.is_empty() {
                    continue;
                }

                handler.emit_err(CompileError::MatchArmVariableNotDefinedInAllAlternatives {
                    match_value: match_value.span.clone(),
                    match_type: engines.help_out(match_value.return_type).to_string(),
                    variable: (*variable).clone(),
                    missing_in_alternatives,
                });
            }

            // Check that the variable types are the same in all alternatives
            // (assuming that the variable exist in the alternative).

            // To the equality, we accept type aliases and the types they encapsulate
            // to be equal, otherwise, we are strict, e.g., no coercion between u8 and u16, etc.
            let equality = UnifyCheck::non_dynamic_equality(engines);

            for (variable, type_id) in variables {
                let type_mismatched_vars =
                    variables_in_alternatives.iter().flat_map(|(_, declarations)| {
                        declarations
                            .iter()
                            .filter(|(ident, decl_expr)| {
                                ident == variable && !equality.check(type_id, decl_expr.return_type)
                            })
                            .map(|(ident, decl_expr)| (ident.clone(), decl_expr.return_type))
                    });

                for type_mismatched_var in type_mismatched_vars {
                    handler.emit_err(CompileError::MatchArmVariableMismatchedType {
                        match_value: match_value.span.clone(),
                        match_type: engines.help_out(match_value.return_type).to_string(),
                        variable: type_mismatched_var.0,
                        first_definition: variable.span(),
                        expected: engines.help_out(type_id).to_string(),
                        received: engines.help_out(type_mismatched_var.1).to_string(),
                    });
                }
            }

            Ok(ReqDeclTree::or(nodes.into_iter().map(|req_decl_tree| req_decl_tree.root).collect()))
        }),
        ty::TyScrutineeVariant::CatchAll => Ok(ReqDeclTree::none()),
        ty::TyScrutineeVariant::Literal(value) => Ok(match_literal(exp, value, span)),
        ty::TyScrutineeVariant::Variable(name) => Ok(match_variable(exp, name)),
        ty::TyScrutineeVariant::Constant(name, _, const_decl) => Ok(match_constant(
            ctx,
            exp,
            name,
            const_decl.type_ascription.type_id,
            span,
        )),
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

fn match_literal(exp: &ty::TyExpression, scrutinee: Literal, span: Span) -> ReqDeclTree {
    let (lhs, rhs) = (
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(scrutinee),
            return_type: exp.return_type,
            span,
        },
    );

    ReqDeclTree::req(lhs, rhs)
}

fn match_variable(exp: &ty::TyExpression, scrutinee_name: Ident) -> ReqDeclTree {
    let decl = (scrutinee_name, exp.to_owned());

    ReqDeclTree::decl(decl)
}

fn match_constant(
    ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    scrutinee_name: Ident,
    scrutinee_type_id: TypeId,
    span: Span,
) -> ReqDeclTree {
    let (lhs, rhs) = (
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::VariableExpression { // TODO-IG: Why not ConstantExpression?
                name: scrutinee_name.clone(),
                span: span.clone(),
                mutability: ty::VariableMutability::Immutable,
                call_path: Some(CallPath::from(scrutinee_name).to_fullpath(ctx.namespace)),
            },
            return_type: scrutinee_type_id,
            span,
        },
    );

    ReqDeclTree::req(lhs, rhs)
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
                let req_decl_tree = matcher(handler, ctx.by_ref(), match_value, &subfield, match_sub_pattern)?;
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
    enum_value_scrutinee: ty::TyScrutinee, // TODO-IG: What's in it if we just match the enum without value?
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
            return_type: type_engine
                .insert(ctx.engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            span: exp.span.clone(),
        },
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(Literal::U64(variant.tag as u64)),
            return_type: type_engine
                .insert(ctx.engines, TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            span: exp.span.clone(),
        },
    );

    nodes.push(ReqDeclNode::req(enum_variant_req.0, enum_variant_req.1));

    // Afterwards, we need to collect the requirements for the enum underlying value.
    let unsafe_downcast = instantiate_enum_unsafe_downcast(exp, variant, call_path_decl, span);
    let req_decl_tree = matcher(handler, ctx, match_value, &unsafe_downcast, enum_value_scrutinee)?;

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
