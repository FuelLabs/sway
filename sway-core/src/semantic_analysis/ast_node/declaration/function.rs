mod function_parameter;
pub use function_parameter::*;

use crate::{error::*, parse_tree::*, semantic_analysis::*, style::*, type_system::*, types::*};
use sha2::{Digest, Sha256};
use sway_types::{
    Function, Ident, JsonABIFunction, JsonTypeApplication, JsonTypeDeclaration, Property, Span,
    Spanned,
};

#[derive(Clone, Debug)]
pub struct TypedFunctionDeclaration {
    pub name: Ident,
    pub body: TypedCodeBlock,
    pub parameters: Vec<TypedFunctionParameter>,
    pub span: Span,
    pub return_type: TypeId,
    pub initial_return_type: TypeId,
    pub(crate) type_parameters: Vec<TypeParameter>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub return_type_span: Span,
    pub(crate) visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub(crate) is_contract_call: bool,
    pub(crate) purity: Purity,
}

impl PartialEq for CompileWrapper<'_, TypedFunctionDeclaration> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        me.name == them.name
            && me.body.wrap(de) == them.body.wrap(de)
            && me.parameters.wrap(de) == them.parameters.wrap(de)
            && look_up_type_id(me.return_type).wrap(de)
                == look_up_type_id(them.return_type).wrap(de)
            && me.type_parameters == them.type_parameters
            && me.visibility == them.visibility
            && me.is_contract_call == them.is_contract_call
            && me.purity == them.purity
    }
}

impl PartialEq for CompileWrapper<'_, Vec<TypedFunctionDeclaration>> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        if me.len() != them.len() {
            return false;
        }
        me.iter()
            .map(|elem| elem.wrap(de))
            .zip(other.inner.iter().map(|elem| elem.wrap(de)))
            .map(|(left, right)| left == right)
            .all(|elem| elem)
    }
}

impl From<&TypedFunctionDeclaration> for TypedAstNode {
    fn from(o: &TypedFunctionDeclaration) -> Self {
        let span = o.span.clone();
        TypedAstNode {
            content: TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                o.clone(),
            )),
            span,
        }
    }
}

impl CopyTypes for TypedFunctionDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.return_type
            .update_type(type_mapping, &self.return_type_span);
        self.body.copy_types(type_mapping);
    }
}

impl Spanned for TypedFunctionDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TypedFunctionDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl ToJsonAbi for TypedFunctionDeclaration {
    type Output = Function;

    fn generate_json_abi(&self) -> Self::Output {
        Function {
            name: self.name.as_str().to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.as_str().to_string(),
                    type_field: x.type_id.json_abi_str(),
                    components: x.type_id.generate_json_abi(),
                    type_arguments: x
                        .type_id
                        .get_type_parameters()
                        .map(|v| v.iter().map(TypeParameter::generate_json_abi).collect()),
                })
                .collect(),
            outputs: vec![Property {
                name: "".to_string(),
                type_field: self.return_type.json_abi_str(),
                components: self.return_type.generate_json_abi(),
                type_arguments: self
                    .return_type
                    .get_type_parameters()
                    .map(|v| v.iter().map(TypeParameter::generate_json_abi).collect()),
            }],
        }
    }
}

impl TypedFunctionDeclaration {
    pub fn type_check(ctx: TypeCheckContext, fn_decl: FunctionDeclaration) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let FunctionDeclaration {
            name,
            body,
            parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            purity,
            ..
        } = fn_decl;
        is_snake_case(&name).ok(&mut warnings, &mut errors);

        // create a namespace for the function
        let mut fn_namespace = ctx.namespace.clone();

        let mut ctx = ctx.scoped(&mut fn_namespace).with_purity(purity);

        // type check the type parameters
        // insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the function parameters
        // insert them into the namespace
        let mut new_parameters = vec![];
        for parameter in parameters.into_iter() {
            new_parameters.push(check!(
                TypedFunctionParameter::type_check(ctx.by_ref(), parameter),
                continue,
                warnings,
                errors
            ));
        }

        // type check the return type
        let initial_return_type = insert_type(return_type);
        let return_type = check!(
            ctx.resolve_type_with_self(
                initial_return_type,
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        // type check the function body
        //
        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = {
            let ctx = ctx
                .by_ref()
                .with_help_text("Function body's return type does not match up with its return type annotation.")
                .with_type_annotation(return_type);
            check!(
                TypedCodeBlock::type_check(ctx, body),
                (
                    TypedCodeBlock { contents: vec![] },
                    insert_type(TypeInfo::ErrorRecovery)
                ),
                warnings,
                errors
            )
        };

        // gather the return statements
        let return_statements: Vec<&TypedExpression> = body
            .contents
            .iter()
            .flat_map(|node| -> Vec<&TypedReturnStatement> { node.gather_return_statements() })
            .map(|TypedReturnStatement { expr, .. }| expr)
            .collect();

        // unify the types of the return statements with the function return type
        for stmt in return_statements {
            let (mut new_warnings, new_errors) = ctx
                .by_ref()
                .with_type_annotation(return_type)
                .with_help_text("Return statement must return the declared function return type.")
                .unify_with_self(stmt.return_type, &stmt.span);
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }

        let function_decl = TypedFunctionDeclaration {
            name,
            body,
            parameters: new_parameters,
            span,
            return_type,
            initial_return_type,
            type_parameters: new_type_parameters,
            return_type_span,
            visibility,
            // if this is for a contract, then it is a contract call
            is_contract_call: ctx.mode() == Mode::ImplAbiFn,
            purity,
        };

        ok(function_decl, warnings, errors)
    }

    /// If there are parameters, join their spans. Otherwise, use the fn name span.
    pub(crate) fn parameters_span(&self) -> Span {
        if !self.parameters.is_empty() {
            self.parameters.iter().fold(
                self.parameters[0].name.span(),
                |acc, TypedFunctionParameter { type_span, .. }| Span::join(acc, type_span.clone()),
            )
        } else {
            self.name.span()
        }
    }

    pub fn to_fn_selector_value_untruncated(&self) -> CompileResult<Vec<u8>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut hasher = Sha256::new();
        let data = check!(
            self.to_selector_name(),
            return err(warnings, errors),
            warnings,
            errors
        );
        hasher.update(data);
        let hash = hasher.finalize();
        ok(hash.to_vec(), warnings, errors)
    }

    /// Converts a [TypedFunctionDeclaration] into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four bytes.
    pub fn to_fn_selector_value(&self) -> CompileResult<[u8; 4]> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let hash = check!(
            self.to_fn_selector_value_untruncated(),
            return err(warnings, errors),
            warnings,
            errors
        );
        // 4 bytes truncation via copying into a 4 byte buffer
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&hash[..4]);
        ok(buf, warnings, errors)
    }

    pub fn to_selector_name(&self) -> CompileResult<String> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let named_params = self
            .parameters
            .iter()
            .map(
                |TypedFunctionParameter {
                     type_id, type_span, ..
                 }| {
                    resolve_type(*type_id, type_span)
                        .expect("unreachable I think?")
                        .to_selector_name(type_span)
                },
            )
            .filter_map(|name| name.ok(&mut warnings, &mut errors))
            .collect::<Vec<String>>();

        ok(
            format!("{}({})", self.name.as_str(), named_params.join(","),),
            warnings,
            errors,
        )
    }

    pub(crate) fn generate_json_abi_function(
        &self,
        types: &mut Vec<JsonTypeDeclaration>,
    ) -> JsonABIFunction {
        // A list of all `JsonTypeDeclaration`s needed for inputs
        let input_types = self
            .parameters
            .iter()
            .map(|x| JsonTypeDeclaration {
                type_id: *x.initial_type_id,
                type_field: x.initial_type_id.get_json_type_str(x.type_id),
                components: x.initial_type_id.get_json_type_components(types, x.type_id),
                type_parameters: x.type_id.get_json_type_parameters(types, x.type_id),
            })
            .collect::<Vec<_>>();

        // The single `JsonTypeDeclaration` needed for the output
        let output_type = JsonTypeDeclaration {
            type_id: *self.initial_return_type,
            type_field: self.initial_return_type.get_json_type_str(self.return_type),
            components: self
                .return_type
                .get_json_type_components(types, self.return_type),
            type_parameters: self
                .return_type
                .get_json_type_parameters(types, self.return_type),
        };

        // Add the new types to `types`
        types.extend(input_types);
        types.push(output_type);

        // Generate the JSON data for the function
        JsonABIFunction {
            name: self.name.as_str().to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| JsonTypeApplication {
                    name: x.name.to_string(),
                    type_id: *x.initial_type_id,
                    type_arguments: x.initial_type_id.get_json_type_arguments(types, x.type_id),
                })
                .collect(),
            output: JsonTypeApplication {
                name: "".to_string(),
                type_id: *self.initial_return_type,
                type_arguments: self
                    .initial_return_type
                    .get_json_type_arguments(types, self.return_type),
            },
        }
    }
}

#[test]
fn test_function_selector_behavior() {
    use crate::type_system::IntegerBits;
    let decl = TypedFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_no_span("foo"),
        body: TypedCodeBlock { contents: vec![] },
        parameters: vec![],
        span: Span::dummy(),
        return_type: 0.into(),
        initial_return_type: 0.into(),
        type_parameters: vec![],
        return_type_span: Span::dummy(),
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "foo()".to_string());

    let decl = TypedFunctionDeclaration {
        purity: Default::default(),
        name: Ident::new_with_override("bar", Span::dummy()),
        body: TypedCodeBlock { contents: vec![] },
        parameters: vec![
            TypedFunctionParameter {
                name: Ident::new_no_span("foo"),
                is_reference: false,
                is_mutable: false,
                type_id: crate::type_system::insert_type(TypeInfo::Str(5)),
                initial_type_id: crate::type_system::insert_type(TypeInfo::Str(5)),
                type_span: Span::dummy(),
            },
            TypedFunctionParameter {
                name: Ident::new_no_span("baz"),
                is_reference: false,
                is_mutable: false,
                type_id: insert_type(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
                initial_type_id: crate::type_system::insert_type(TypeInfo::Str(5)),
                type_span: Span::dummy(),
            },
        ],
        span: Span::dummy(),
        return_type: 0.into(),
        initial_return_type: 0.into(),
        type_parameters: vec![],
        return_type_span: Span::dummy(),
        visibility: Visibility::Public,
        is_contract_call: false,
    };

    let selector_text = match decl.to_selector_name().value {
        Some(value) => value,
        _ => panic!("test failure"),
    };

    assert_eq!(selector_text, "bar(str[5],u32)".to_string());
}
