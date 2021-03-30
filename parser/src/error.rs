use crate::parser::Rule;
use crate::types::TypeInfo;
use inflector::cases::classcase::to_class_case;
use inflector::cases::snakecase::to_snake_case;
use pest::Span;
use thiserror::Error;

macro_rules! type_check {
    ($fn_expr: expr, $err_recov: expr, $warnings: ident, $errors: ident) => {{
        use crate::CompileResult;
        let res = $fn_expr;
        match res {
            CompileResult::Ok {
                value,
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                $warnings.append(&mut l_w);
                $errors.append(&mut l_e);
                value
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                $warnings.append(&mut l_w);
                $errors.append(&mut l_e);
                $err_recov
            }
        }
    }};
}

/// evaluates `$fn` with argument `$arg`, and pushes any warnings to the `$warnings` buffer.
macro_rules! eval {
    ($fn: expr, $warnings: ident, $errors: ident, $arg: expr, $error_recovery: expr) => {{
        use crate::CompileResult;
        let res = match $fn($arg.clone()) {
            CompileResult::Ok {
                value,
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                $warnings.append(&mut l_w);
                $errors.append(&mut l_e);
                value
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                $errors.append(&mut l_e);
                $warnings.append(&mut l_w);
                $error_recovery
            }
        };
        res
    }};
}

macro_rules! assert_or_warn {
    ($bool_expr: expr, $warnings: ident, $span: expr, $warning: expr) => {
        if !$bool_expr {
            use crate::error::CompileWarning;
            $warnings.push(CompileWarning {
                warning_content: $warning,
                span: $span,
            });
        }
    };
}

/// Denotes a non-recoverable state
pub(crate) fn err<'sc, T>(
    warnings: Vec<CompileWarning<'sc>>,
    errors: Vec<CompileError<'sc>>,
) -> CompileResult<'sc, T> {
    CompileResult::Err { warnings, errors }
}

/// Denotes a recovered or non-error state
pub(crate) fn ok<'sc, T>(
    value: T,
    warnings: Vec<CompileWarning<'sc>>,
    errors: Vec<CompileError<'sc>>,
) -> CompileResult<'sc, T> {
    CompileResult::Ok {
        warnings,
        value,
        errors,
    }
}

#[derive(Debug)]
pub enum CompileResult<'sc, T> {
    Ok {
        value: T,
        warnings: Vec<CompileWarning<'sc>>,
        errors: Vec<CompileError<'sc>>,
    },
    Err {
        warnings: Vec<CompileWarning<'sc>>,
        errors: Vec<CompileError<'sc>>,
    },
}

impl<'sc, T> CompileResult<'sc, T> {
    pub fn unwrap(&self) -> &T {
        match self {
            CompileResult::Ok { value, .. } => value,
            CompileResult::Err { errors, .. } => {
                panic!("Unwrapped an err {:?}", errors);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileWarning<'sc> {
    pub span: Span<'sc>,
    pub warning_content: Warning<'sc>,
}

impl<'sc> CompileWarning<'sc> {
    pub fn to_friendly_warning_string(&self) -> String {
        self.warning_content.to_string()
    }

    pub fn span(&self) -> (usize, usize) {
        (self.span.start(), self.span.end())
    }
}

#[derive(Debug, Clone)]
pub enum Warning<'sc> {
    NonClassCaseStructName {
        struct_name: &'sc str,
    },
    NonClassCaseTraitName {
        name: &'sc str,
    },
    NonClassCaseEnumName {
        enum_name: &'sc str,
    },
    NonClassCaseEnumVariantName {
        variant_name: &'sc str,
    },
    NonSnakeCaseStructFieldName {
        field_name: &'sc str,
    },
    NonSnakeCaseFunctionName {
        name: &'sc str,
    },
    LossOfPrecision {
        initial_type: TypeInfo<'sc>,
        cast_to: TypeInfo<'sc>,
    },
    UnusedReturnValue {
        r#type: TypeInfo<'sc>,
    },
    SimilarMethodFound {
        lib: String,
        module: String,
        name: String,
    },
    OverridesOtherSymbol {
        name: &'sc str,
    },
    OverridingTraitImplementation,
}

impl<'sc> Warning<'sc> {
    fn to_string(&self) -> String {
        use Warning::*;
        match self {
            NonClassCaseStructName{ struct_name } => format!("Struct name \"{}\" is not idiomatic. Structs should have a ClassCase name, like \"{}\".", struct_name, to_class_case(struct_name)),
            NonClassCaseTraitName{ name } => format!("Trait name \"{}\" is not idiomatic. Traits should have a ClassCase name, like \"{}\".", name, to_class_case(name)),
            NonClassCaseEnumName{ enum_name} => format!("Enum \"{}\"'s capitalization is not idiomatic. Enums should have a ClassCase name, like \"{}\".", enum_name, to_class_case(enum_name)),
            NonSnakeCaseStructFieldName { field_name } => format!("Struct field name \"{}\" is not idiomatic. Struct field names should have a snake_case name, like \"{}\".", field_name, to_snake_case(field_name)),
            NonClassCaseEnumVariantName { variant_name } => format!("Enum variant name \"{}\" is not idiomatic. Enum variant names should be ClassCase, like \"{}\".", variant_name, to_class_case(variant_name)),
            NonSnakeCaseFunctionName { name } => format!("Function name \"{}\" is not idiomatic. Function names should be snake_case, like \"{}\".", name, to_snake_case(name)),
            LossOfPrecision { initial_type, cast_to } => format!("This cast, from type {} to type {}, will lose precision.", initial_type.friendly_type_str(), cast_to.friendly_type_str()),
            UnusedReturnValue { r#type } => format!("This returns a value of type {}, which is not assigned to anything and is ignored.", r#type.friendly_type_str()),
            SimilarMethodFound { lib, module, name } => format!("A method with the same name was found for type {} in dependency \"{}::{}\". Traits must be in scope in order to access their methods. ", name, lib, module),
            OverridesOtherSymbol { name } => format!("This import would override another symbol with the same name \"{}\" in this namespace.", name),
            OverridingTraitImplementation  => format!("This trait implementation overrides another one that was previously defined.")
        }
    }
}

#[derive(Error, Debug)]
pub enum CompileError<'sc> {
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: &'sc str, span: Span<'sc> },
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariablePath { var_name: String, span: Span<'sc> },
    #[error("Function \"{name}\" does not exist in this scope.")]
    UnknownFunction { name: &'sc str, span: Span<'sc> },
    #[error("Identifier \"{name}\" was used as a variable, but it is actually a {what_it_is}.")]
    NotAVariable {
        name: &'sc str,
        span: Span<'sc>,
        what_it_is: &'static str,
    },
    #[error("Identifier \"{name}\" was called as if it was a function, but it is actually a {what_it_is}.")]
    NotAFunction {
        name: &'sc str,
        span: Span<'sc>,
        what_it_is: &'static str,
    },
    #[error("Unimplemented feature: {0}")]
    Unimplemented(&'static str, Span<'sc>),
    #[error("{0}")]
    TypeError(TypeError<'sc>),
    #[error("Error parsing input: expected {0:?}")]
    ParseFailure(#[from] pest::error::Error<Rule>),
    #[error("Invalid top-level item: {0:?}. A program should consist of a contract, script, or predicate at the top level.")]
    InvalidTopLevelItem(Rule, Span<'sc>),
    #[error("Internal compiler error: {0}\nPlease file an issue on the repository and include the code that triggered this error.")]
    Internal(&'static str, Span<'sc>),
    #[error("Unimplemented feature: {0:?}")]
    UnimplementedRule(Rule, Span<'sc>),
    #[error("Byte literal had length of {byte_length}. Byte literals must be either one byte long (8 binary digits or 2 hex digits) or 32 bytes long (256 binary digits or 64 hex digits)")]
    InvalidByteLiteralLength { byte_length: usize, span: Span<'sc> },
    #[error("Expected an expression to follow operator \"{op}\"")]
    ExpectedExprAfterOp { op: &'sc str, span: Span<'sc> },
    #[error("Expected an operator, but \"{op}\" is not a recognized operator. ")]
    ExpectedOp { op: &'sc str, span: Span<'sc> },
    #[error("Where clause was specified but there are no generic type parameters. Where clauses can only be applied to generic type parameters.")]
    UnexpectedWhereClause(Span<'sc>),
    #[error("Specified generic type in where clause \"{type_name}\" not found in generic type arguments of function.")]
    UndeclaredGenericTypeInWhereClause {
        type_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Program contains multiple contracts. A valid program should only contain at most one contract.")]
    MultipleContracts(Span<'sc>),
    #[error("Program contains multiple scripts. A valid program should only contain at most one script.")]
    MultipleScripts(Span<'sc>),
    #[error("Program contains multiple predicates. A valid program should only contain at most one predicate.")]
    MultiplePredicates(Span<'sc>),
    #[error("Trait constraint was applied to generic type that is not in scope. Trait \"{trait_name}\" cannot constrain type \"{type_name}\" because that type does not exist in this scope.")]
    ConstrainedNonExistentType {
        trait_name: &'sc str,
        type_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Predicate definition contains multiple main functions. Multiple functions in the same scope cannot have the same name.")]
    MultiplePredicateMainFunctions(Span<'sc>),
    #[error(
        "Predicate declaration contains no main function. Predicates require a main function."
    )]
    NoPredicateMainFunction(Span<'sc>),
    #[error("A predicate's main function must return a boolean.")]
    PredicateMainDoesNotReturnBool(Span<'sc>),
    #[error("Script declaration contains no main function. Scripts require a main function.")]
    NoScriptMainFunction(Span<'sc>),
    #[error("Script definition contains multiple main functions. Multiple functions in the same scope cannot have the same name.")]
    MultipleScriptMainFunctions(Span<'sc>),
    #[error("Attempted to reassign to a symbol that is not a variable. Symbol {name} is not a mutable variable, it is a {kind}.")]
    ReassignmentToNonVariable {
        name: &'sc str,
        kind: &'sc str,
        span: Span<'sc>,
    },
    #[error("Assignment to immutable variable. Variable {0} is not declared as mutable.")]
    AssignmentToNonMutable(&'sc str, Span<'sc>),
    #[error("Generic type \"{name}\" is not in scope. Perhaps you meant to specify type parameters in the function signature? For example: \n`fn {fn_name}<{comma_separated_generic_params}>({args}) -> ... `")]
    TypeParameterNotInTypeScope {
        name: &'sc str,
        span: Span<'sc>,
        comma_separated_generic_params: String,
        fn_name: &'sc str,
        args: &'sc str,
    },
    #[error(
        "Asm opcode has multiple immediates specified, when any opcode has at most one immediate."
    )]
    MultipleImmediates(Span<'sc>),
    #[error(
        "Expected type {expected}, but found type {given}. The definition of this function must match the one in the trait declaration."
    )]
    MismatchedTypeInTrait {
        span: Span<'sc>,
        given: String,
        expected: String,
    },
    #[error("\"{name}\" is not a trait, so it cannot be \"impl'd\". ")]
    NotATrait { span: Span<'sc>, name: &'sc str },
    #[error("Trait \"{name}\" cannot be found in the current scope.")]
    UnknownTrait { span: Span<'sc>, name: &'sc str },
    #[error("Function \"{name}\" is not a part of trait \"{trait_name}\"'s interface surface.")]
    FunctionNotAPartOfInterfaceSurface {
        name: &'sc str,
        trait_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Functions are missing from this trait implementation: {missing_functions}")]
    MissingInterfaceSurfaceMethods {
        missing_functions: String,
        span: Span<'sc>,
    },
    #[error("Expected {expected} type arguments, but instead found {given}.")]
    IncorrectNumberOfTypeArguments {
        given: usize,
        expected: usize,
        span: Span<'sc>,
    },
    #[error("Struct with name \"{name}\" could not be found in this scope. Perhaps you need to import it?")]
    StructNotFound { name: &'sc str, span: Span<'sc> },
    #[error("The name \"{name}\" does not refer to a struct, but this is an attempted struct declaration.")]
    DeclaredNonStructAsStruct { name: &'sc str, span: Span<'sc> },
    #[error("Attempted to access field \"{field_name}\" of non-struct \"{name}\". Field accesses are only valid on structs.")]
    AccessedFieldOfNonStruct {
        field_name: &'sc str,
        name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Attempted to access a method on something that has no methods. \"{name}\" is a {thing}, not a type with methods.")]
    MethodOnNonValue {
        name: &'sc str,
        thing: &'sc str,
        span: Span<'sc>,
    },
    #[error("Initialization of struct \"{struct_name}\" is missing field \"{field_name}\".")]
    StructMissingField {
        field_name: &'sc str,
        struct_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Struct \"{struct_name}\" does not have field \"{field_name}\".")]
    StructDoesntHaveThisField {
        field_name: &'sc str,
        struct_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("No method named {method_name} found for type {type_name}.")]
    MethodNotFound {
        span: Span<'sc>,
        method_name: &'sc str,
        type_name: String,
    },
    #[error("The asterisk, if present, must be the last part of a path. E.g., `use foo::bar::*`.")]
    NonFinalAsteriskInPath { span: Span<'sc> },
    #[error("Module \"{name}\" could not be found.")]
    ModuleNotFound { span: Span<'sc>, name: String },
    #[error("\"{name}\" is not a struct. Fields can only be accessed on structs.")]
    NotAStruct { name: &'sc str, span: Span<'sc> },
    #[error("Field \"{field_name}\" not found on struct \"{struct_name}\". Available fields are:\n {available_fields}")]
    FieldNotFound {
        field_name: &'sc str,
        available_fields: String,
        struct_name: &'sc str,
        span: Span<'sc>,
    },
    #[error("Could not find symbol {name} in this scope.")]
    SymbolNotFound { span: Span<'sc>, name: &'sc str },
    #[error("Because this if expression's value is used, an \"else\" branch is required and it must return type \"{r#type}\"")]
    NoElseBranch { span: Span<'sc>, r#type: String },
}

impl<'sc> std::convert::From<TypeError<'sc>> for CompileError<'sc> {
    fn from(other: TypeError<'sc>) -> CompileError<'sc> {
        CompileError::TypeError(other)
    }
}

#[derive(Error, Debug)]
pub enum TypeError<'sc> {
    #[error("Mismatched types: Expected type {expected} but found type {received}. Type {received} is not castable to type {expected}.\n help: {help_text}")]
    MismatchedType {
        expected: String,
        received: String,
        help_text: String,
        span: Span<'sc>,
    },
}

impl<'sc> TypeError<'sc> {
    pub(crate) fn span(&self) -> (usize, usize) {
        use TypeError::*;
        match self {
            MismatchedType { span, .. } => (span.start(), span.end()),
        }
    }
}

impl<'sc> CompileError<'sc> {
    pub fn to_friendly_error_string(&self) -> String {
        match self {
            CompileError::ParseFailure(err) => format!(
                "Error parsing input: {}",
                match &err.variant {
                    pest::error::ErrorVariant::ParsingError {
                        positives,
                        negatives,
                    } => {
                        let mut buf = String::new();
                        if !positives.is_empty() {
                            buf.push_str(&format!(
                                "expected one of [{}]",
                                positives
                                    .iter()
                                    .map(|x| format!("{:?}", x))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        if !negatives.is_empty() {
                            buf.push_str(&format!(
                                "did not expect any of [{}]",
                                negatives
                                    .iter()
                                    .map(|x| format!("{:?}", x))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        buf
                    }
                    pest::error::ErrorVariant::CustomError { message } => message.to_string(),
                }
            ),
            a => format!("{}", a),
        }
    }

    pub fn span(&self) -> (usize, usize) {
        use CompileError::*;
        match self {
            UnknownVariable { span, .. } => (span.start(), span.end()),
            UnknownVariablePath { span, .. } => (span.start(), span.end()),
            UnknownFunction { span, .. } => (span.start(), span.end()),
            NotAVariable { span, .. } => (span.start(), span.end()),
            NotAFunction { span, .. } => (span.start(), span.end()),
            Unimplemented(_, span) => (span.start(), span.end()),
            TypeError(err) => err.span(),
            ParseFailure(err) => match err.location {
                pest::error::InputLocation::Pos(num) => (num, num + 1),
                pest::error::InputLocation::Span((start, end)) => (start, end),
            },
            InvalidTopLevelItem(_, sp) => (sp.start(), sp.end()),
            Internal(_, sp) => (sp.start(), sp.end()),
            UnimplementedRule(_, sp) => (sp.start(), sp.end()),
            InvalidByteLiteralLength { span, .. } => (span.start(), span.end()),
            ExpectedExprAfterOp { span, .. } => (span.start(), span.end()),
            ExpectedOp { span, .. } => (span.start(), span.end()),
            UnexpectedWhereClause(sp) => (sp.start(), sp.end()),
            UndeclaredGenericTypeInWhereClause { span, .. } => (span.start(), span.end()),
            MultiplePredicates(sp) => (sp.start(), sp.end()),
            MultipleScripts(sp) => (sp.start(), sp.end()),
            MultipleContracts(sp) => (sp.start(), sp.end()),
            ConstrainedNonExistentType { span, .. } => (span.start(), span.end()),
            MultiplePredicateMainFunctions(sp) => (sp.start(), sp.end()),
            NoPredicateMainFunction(sp) => (sp.start(), sp.end()),
            PredicateMainDoesNotReturnBool(sp) => (sp.start(), sp.end()),
            NoScriptMainFunction(sp) => (sp.start(), sp.end()),
            MultipleScriptMainFunctions(sp) => (sp.start(), sp.end()),
            ReassignmentToNonVariable { span, .. } => (span.start(), span.end()),
            AssignmentToNonMutable(_, sp) => (sp.start(), sp.end()),
            TypeParameterNotInTypeScope { span, .. } => (span.start(), span.end()),
            MultipleImmediates(sp) => (sp.start(), sp.end()),
            MismatchedTypeInTrait { span, .. } => (span.start(), span.end()),
            NotATrait { span, .. } => (span.start(), span.end()),
            UnknownTrait { span, .. } => (span.start(), span.end()),
            FunctionNotAPartOfInterfaceSurface { span, .. } => (span.start(), span.end()),
            MissingInterfaceSurfaceMethods { span, .. } => (span.start(), span.end()),
            IncorrectNumberOfTypeArguments { span, .. } => (span.start(), span.end()),
            StructNotFound { span, .. } => (span.start(), span.end()),
            DeclaredNonStructAsStruct { span, .. } => (span.start(), span.end()),
            AccessedFieldOfNonStruct { span, .. } => (span.start(), span.end()),
            MethodOnNonValue { span, .. } => (span.start(), span.end()),
            StructMissingField { span, .. } => (span.start(), span.end()),
            StructDoesntHaveThisField { span, .. } => (span.start(), span.end()),
            MethodNotFound { span, .. } => (span.start(), span.end()),
            NonFinalAsteriskInPath { span, .. } => (span.start(), span.end()),
            ModuleNotFound { span, .. } => (span.start(), span.end()),
            NotAStruct { span, .. } => (span.start(), span.end()),
            FieldNotFound { span, .. } => (span.start(), span.end()),
            SymbolNotFound { span, .. } => (span.start(), span.end()),
            NoElseBranch { span, .. } => (span.start(), span.end()),
        }
    }
}
