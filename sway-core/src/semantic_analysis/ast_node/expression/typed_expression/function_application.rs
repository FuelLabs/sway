use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    error::*,
    semantic_analysis::{ast_node::*, TCOpts, TypeCheckArguments},
    type_engine::TypeId,
};
use std::collections::{hash_map::RandomState, HashMap, VecDeque};

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    function_decl: TypedFunctionDeclaration,
    call_path: CallPath,
    type_arguments: Vec<TypeArgument>,
    arguments: Vec<Expression>,
    namespace: &mut Namespace,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // monomorphize the function declaration
    let function_decl = check!(
        namespace.monomorphize(
            function_decl,
            type_arguments,
            EnforceTypeArguments::No,
            Some(self_type),
            Some(&call_path.span())
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    // 'purity' is that of the callee, 'opts.purity' of the caller.
    if !opts.purity.can_call(function_decl.purity) {
        errors.push(CompileError::StorageAccessMismatch {
            attrs: promote_purity(opts.purity, function_decl.purity).to_attribute_syntax(),
            span: function_decl.name.span().clone(),
        });
    }

    // type check arguments in function application vs arguments in function
    // declaration. Use parameter type annotations as annotations for the
    // arguments
    let typed_call_arguments = arguments
        .into_iter()
        .zip(function_decl.parameters.iter())
        .map(|(arg, param)| {
            let exp = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: arg.clone(),
                    namespace,
                    return_type_annotation: param.r#type,
                    help_text: "The argument that has been provided to this function's type does \
                        not match the declared type of the parameter in the function \
                        declaration.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
                error_recovery_expr(arg.span()),
                warnings,
                errors
            );
            (param.name.clone(), exp)
        })
        .collect();

    let span = function_decl.span.clone();
    let exp = check!(
        instantiate_function_application_inner(
            call_path,
            HashMap::new(),
            typed_call_arguments,
            function_decl,
            None,
            IsConstant::No,
            span,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(exp, warnings, errors)
}

pub(crate) fn instantiate_function_application_simple(
    call_path: CallPath,
    contract_call_params: HashMap<String, TypedExpression, RandomState>,
    arguments: VecDeque<TypedExpression>,
    function_decl: TypedFunctionDeclaration,
    selector: Option<ContractCallMetadata>,
    is_constant: IsConstant,
    span: Span,
) -> CompileResult<TypedExpression> {
    let args_and_names = function_decl
        .parameters
        .iter()
        .zip(arguments.into_iter())
        .map(|(param, arg)| (param.name.clone(), arg))
        .collect::<Vec<(_, _)>>();
    instantiate_function_application_inner(
        call_path,
        contract_call_params,
        args_and_names,
        function_decl,
        selector,
        is_constant,
        span,
    )
}

#[allow(clippy::comparison_chain)]
fn instantiate_function_application_inner(
    call_path: CallPath,
    contract_call_params: HashMap<String, TypedExpression, RandomState>,
    arguments: Vec<(Ident, TypedExpression)>,
    function_decl: TypedFunctionDeclaration,
    selector: Option<ContractCallMetadata>,
    is_constant: IsConstant,
    span: Span,
) -> CompileResult<TypedExpression> {
    let warnings = vec![];
    let mut errors = vec![];
    match arguments.len().cmp(&function_decl.parameters.len()) {
        std::cmp::Ordering::Equal => {
            let exp = TypedExpression {
                expression: TypedExpressionVariant::FunctionApplication {
                    call_path,
                    contract_call_params,
                    arguments,
                    function_body: function_decl.body.clone(),
                    selector,
                },
                return_type: function_decl.return_type,
                is_constant,
                span,
            };
            ok(exp, warnings, errors)
        }
        std::cmp::Ordering::Less => {
            errors.push(CompileError::TooFewArgumentsForFunction {
                span,
                method_name: function_decl.name,
                expected: function_decl.parameters.len(),
                received: arguments.len(),
            });
            err(warnings, errors)
        }
        std::cmp::Ordering::Greater => {
            errors.push(CompileError::TooManyArgumentsForFunction {
                span,
                method_name: function_decl.name,
                expected: function_decl.parameters.len(),
                received: arguments.len(),
            });
            err(warnings, errors)
        }
    }
}
