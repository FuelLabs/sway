use crate::parser::Rule;
use crate::types::TypeInfo;
use inflector::cases::classcase::to_class_case;
use inflector::cases::snakecase::to_snake_case;
use pest::Span;
use thiserror::Error;

macro_rules! type_check {
    ($name: ident, $val: expr, $namespace: expr, $type_annotation: expr, $help_text: expr, $err_recov: expr, $warnings: ident, $errors: ident) => {{
        use crate::CompileResult;
        let res = $name::type_check($val.clone(), $namespace, $type_annotation, $help_text);
        match res {
            CompileResult::Ok { value, warnings: mut l_w } => {
                $warnings.append(&mut l_w);
                value
            },
            CompileResult::Err {
                warnings: mut l_w, errors: mut l_e
            } => {
                $warnings.append(&mut l_w);
                $errors.append(&mut l_e);
                $err_recov
            }
        }
    }}
}

/// evaluates `$fn` with argument `$arg`, and pushes any warnings to the `$warnings` buffer.
macro_rules! eval {
    ($fn: expr, $warnings: ident, $errors: ident, $arg: expr, $error_recovery: expr) => {{
        use crate::CompileResult;
        let  res = match $fn($arg.clone()) {
            CompileResult::Ok { value, warnings: mut l_w, errors: mut l_e } => {
                $warnings.append(&mut l_w);
                $errors.append(&mut l_e);
                value
            },
            CompileResult::Err {  warnings: mut l_w,  errors: mut l_e }   => {
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
pub(crate) fn err<'sc, T>(warnings: Vec<CompileWarning<'sc>>, errors: Vec<CompileError<'sc>>) -> CompileResult<'sc, T> {
    CompileResult::Err { warnings, errors }
}

/// Denotes a recovered or non-error state
pub(crate) fn ok<T>(value: T, warnings: Vec<CompileWarning>, errors: Vec<CompileError<'sc>>) -> CompileResult<T> {
    CompileResult::Ok { warnings, value, errors }
}

#[derive(Debug)]
pub enum CompileResult<'sc, T> {
    Ok {
        value: T,
        warnings: Vec<CompileWarning<'sc>>,
    },
    Err {
        warnings: Vec<CompileWarning<'sc>>,
        errors: Vec<CompileError<'sc>>,
    },
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
}

impl<'sc> Warning<'sc> {
    fn to_string(&self) -> String {
        use Warning::*;
        match self {
            NonClassCaseStructName{ struct_name } => format!("Struct name \"{}\" is not idiomatic. Structs should have a ClassCase name, like \"{}\".", struct_name, to_class_case(struct_name)),
            NonClassCaseEnumName{ enum_name} => format!("Enum \"{}\"'s capitalization is not idiomatic. Enums should have a ClassCase name, like \"{}\".", enum_name, to_class_case(enum_name)),
            NonSnakeCaseStructFieldName { field_name } => format!("Struct field name \"{}\" is not idiomatic. Struct field names should have a snake_case name, like \"{}\".", field_name, to_snake_case(field_name)),
            NonClassCaseEnumVariantName { variant_name } => format!("Enum variant name \"{}\" is not idiomatic. Enum variant names should be ClassCase, like \"{}\".", variant_name, to_class_case(variant_name)),
            NonSnakeCaseFunctionName { name } => format!("Function name \"{}\" is not idiomatic. Function names should be snake_case, like \"{}\".", name, to_snake_case(name)),
            LossOfPrecision { initial_type, cast_to } => format!("This cast, from type {} to type {}, will lose precision.", initial_type.friendly_type_str(), cast_to.friendly_type_str()),
        }
    }
}

#[derive(Error, Debug)]
pub enum CompileError<'sc> {
    #[error("Variable \"{var_name}\" does not exist in this scope.")]
    UnknownVariable { var_name: &'sc str, span: Span<'sc> },
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
    ConstrainedNonExistentType{ trait_name: &'sc str, type_name: &'sc str, span: Span<'sc> }
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
        use CompileError::*;
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
            ConstrainedNonExistentType { span, .. } => (span.start(), span.end())
        }
    }
}
