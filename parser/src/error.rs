use crate::parser::Rule;
use inflector::cases::classcase::to_class_case;
use inflector::cases::snakecase::to_snake_case;
use pest::Span;
use thiserror::Error;

/// evaluates `$fn` with argument `$arg`, and pushes any warnings to the `$warnings` buffer.
macro_rules! eval {
    ($fn: expr, $warnings: ident, $arg: expr) => {{
        let (res, mut warns) = $fn($arg)?;
        $warnings.append(&mut warns);
        res
    }};
}

macro_rules! assert_or_warn {
    ($bool_expr: expr, $warnings: ident, $span: expr, $warning: expr) => {
        if !$bool_expr {
            $warnings.push(CompileWarning {
                warning_content: $warning,
                span: $span,
            });
        }
    };
}

pub type CompileResult<'sc, T> = Result<(T, Vec<CompileWarning<'sc>>), CompileError<'sc>>;

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
    NonClassCaseStructName { struct_name: &'sc str },
    NonSnakeCaseStructFieldName { field_name: &'sc str },
}

impl<'sc> Warning<'sc> {
    fn to_string(&self) -> String {
        use Warning::*;
        match self {
            NonClassCaseStructName{ struct_name } => format!("Struct \"{}\"'s capitalization is not idiomatic. Structs should have a ClassCase name, like \"{}\".", struct_name, to_class_case(struct_name)),
            NonSnakeCaseStructFieldName { field_name } => format!("Struct field name \"{}\" is not idiomatic. Struct field names should have a snake_case name, like \"{}\".", field_name, to_snake_case(field_name)),
        }
    }
}

#[derive(Debug, Error)]
pub enum CompileError<'sc> {
    #[error("Error parsing input: expected {0:?}")]
    ParseFailure(#[from] pest::error::Error<Rule>),
    #[error("Invalid top-level item: {0:?}. A program should consist of a contract, script, or predicate at the top level.")]
    InvalidTopLevelItem(Rule, Span<'sc>),
    #[error("Internal compiler error: {0}. Please file an issue on the repository and include the code that triggered this error.")]
    Internal(&'static str, Span<'sc>),
    #[error("Unimplemented feature: {0:?}")]
    Unimplemented(Rule, Span<'sc>),
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
}

impl<'sc> CompileError<'sc> {
    pub fn span(&self) -> (usize, usize) {
        use CompileError::*;
        match self {
            ParseFailure(err) => match err.location {
                pest::error::InputLocation::Pos(num) => (num, num + 1),
                pest::error::InputLocation::Span((start, end)) => (start, end),
            },
            InvalidTopLevelItem(_, sp) => (sp.start(), sp.end()),
            Internal(_, sp) => (sp.start(), sp.end()),
            Unimplemented(_, sp) => (sp.start(), sp.end()),
            InvalidByteLiteralLength { span, .. } => (span.start(), span.end()),
            ExpectedExprAfterOp { span, .. } => (span.start(), span.end()),
            ExpectedOp { span, .. } => (span.start(), span.end()),
            UnexpectedWhereClause(sp) => (sp.start(), sp.end()),
            UndeclaredGenericTypeInWhereClause { span, .. } => (span.start(), span.end()),
        }
    }

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
            o => format!("{}", o),
        }
    }
}
