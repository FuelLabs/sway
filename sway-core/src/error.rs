//! Tools related to handling/recovering from Sway compile errors and reporting them to the user.

use crate::{
    language::parsed::VariableDeclaration,
    style::{to_screaming_snake_case, to_snake_case, to_upper_camel_case},
    type_system::*,
};
use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{fmt, path::PathBuf, sync::Arc};

macro_rules! check {
    ($fn_expr: expr, $error_recovery: expr, $warnings: ident, $errors: ident $(,)?) => {{
        let mut res = $fn_expr;
        $warnings.append(&mut res.warnings);
        $errors.append(&mut res.errors);
        #[allow(clippy::manual_unwrap_or)]
        match res.value {
            None => $error_recovery,
            Some(value) => value,
        }
    }};
}

macro_rules! append {
    ($fn_expr: expr, $warnings: ident, $errors: ident $(,)?) => {{
        let (mut l, mut r) = $fn_expr;
        $warnings.append(&mut l);
        $errors.append(&mut r);
    }};
}

macro_rules! assert_or_warn {
    ($bool_expr: expr, $warnings: ident, $span: expr, $warning: expr $(,)?) => {{
        if !$bool_expr {
            use crate::error::CompileWarning;
            $warnings.push(CompileWarning {
                warning_content: $warning,
                span: $span,
            });
        }
    }};
}

/// Denotes a non-recoverable state
pub(crate) fn err<T>(warnings: Vec<CompileWarning>, errors: Vec<CompileError>) -> CompileResult<T> {
    CompileResult {
        value: None,
        warnings,
        errors,
    }
}

/// Denotes a recovered or non-error state
pub(crate) fn ok<T>(
    value: T,
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
) -> CompileResult<T> {
    CompileResult {
        value: Some(value),
        warnings,
        errors,
    }
}

/// Acts as the result of parsing `Declaration`s, `Expression`s, etc.
/// Some `Expression`s need to be able to create `VariableDeclaration`s,
/// so this struct is used to "bubble up" those declarations to a viable
/// place in the AST.
#[derive(Debug, Clone)]
pub struct ParserLifter<T> {
    pub var_decls: Vec<VariableDeclaration>,
    pub value: T,
}

impl<T> ParserLifter<T> {
    #[allow(dead_code)]
    pub(crate) fn empty(value: T) -> Self {
        ParserLifter {
            var_decls: vec![],
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult<T> {
    pub value: Option<T>,
    pub warnings: Vec<CompileWarning>,
    pub errors: Vec<CompileError>,
}

impl<T> From<Result<T, CompileError>> for CompileResult<T> {
    fn from(o: Result<T, CompileError>) -> Self {
        match o {
            Ok(o) => CompileResult {
                value: Some(o),
                warnings: vec![],
                errors: vec![],
            },
            Err(e) => CompileResult {
                value: None,
                warnings: vec![],
                errors: vec![e],
            },
        }
    }
}

impl<T> CompileResult<T> {
    pub fn is_ok(&self) -> bool {
        self.value.is_some() && self.errors.is_empty()
    }

    pub fn is_ok_no_warn(&self) -> bool {
        self.value.is_some() && self.warnings.is_empty() && self.errors.is_empty()
    }

    pub fn new(value: Option<T>, warnings: Vec<CompileWarning>, errors: Vec<CompileError>) -> Self {
        CompileResult {
            value,
            warnings,
            errors,
        }
    }

    pub fn ok(
        mut self,
        warnings: &mut Vec<CompileWarning>,
        errors: &mut Vec<CompileError>,
    ) -> Option<T> {
        warnings.append(&mut self.warnings);
        errors.append(&mut self.errors);
        self.value
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> CompileResult<U> {
        match self.value {
            None => err(self.warnings, self.errors),
            Some(value) => ok(f(value), self.warnings, self.errors),
        }
    }

    pub fn flat_map<U, F: FnOnce(T) -> CompileResult<U>>(self, f: F) -> CompileResult<U> {
        match self.value {
            None => err(self.warnings, self.errors),
            Some(value) => {
                let res = f(value);
                CompileResult {
                    value: res.value,
                    warnings: [self.warnings, res.warnings].concat(),
                    errors: [self.errors, res.errors].concat(),
                }
            }
        }
    }

    pub fn unwrap(self, warnings: &mut Vec<CompileWarning>, errors: &mut Vec<CompileError>) -> T {
        let panic_msg = format!("Unwrapped an err {:?}", self.errors);
        self.unwrap_or_else(warnings, errors, || panic!("{}", panic_msg))
    }

    pub fn unwrap_or_else<F: FnOnce() -> T>(
        self,
        warnings: &mut Vec<CompileWarning>,
        errors: &mut Vec<CompileError>,
        or_else: F,
    ) -> T {
        self.ok(warnings, errors).unwrap_or_else(or_else)
    }
}

impl<'a, T> CompileResult<&'a T>
where
    T: Clone,
{
    /// Converts a `CompileResult` around a reference value to an owned value by cloning the type
    /// behind the reference.
    pub fn cloned(self) -> CompileResult<T> {
        let CompileResult {
            value,
            warnings,
            errors,
        } = self;
        let value = value.cloned();
        CompileResult {
            value,
            warnings,
            errors,
        }
    }
}

// TODO: since moving to using Idents instead of strings the warning_content will usually contain a
// duplicate of the span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompileWarning {
    pub span: Span,
    pub warning_content: Warning,
}

impl Spanned for CompileWarning {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl CompileWarning {
    pub fn to_friendly_warning_string(&self) -> String {
        self.warning_content.to_string()
    }

    pub fn path(&self) -> Option<Arc<PathBuf>> {
        self.span.path().cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Warning {
    NonClassCaseStructName {
        struct_name: Ident,
    },
    NonClassCaseTypeParameter {
        name: Ident,
    },
    NonClassCaseTraitName {
        name: Ident,
    },
    NonClassCaseEnumName {
        enum_name: Ident,
    },
    NonClassCaseEnumVariantName {
        variant_name: Ident,
    },
    NonSnakeCaseStructFieldName {
        field_name: Ident,
    },
    NonSnakeCaseFunctionName {
        name: Ident,
    },
    NonScreamingSnakeCaseConstName {
        name: Ident,
    },
    LossOfPrecision {
        initial_type: IntegerBits,
        cast_to: IntegerBits,
    },
    UnusedReturnValue {
        r#type: Box<TypeInfo>,
    },
    SimilarMethodFound {
        lib: Ident,
        module: Ident,
        name: Ident,
    },
    ShadowsOtherSymbol {
        name: Ident,
    },
    OverridingTraitImplementation,
    DeadDeclaration,
    DeadFunctionDeclaration,
    DeadStructDeclaration,
    DeadTrait,
    UnreachableCode,
    DeadEnumVariant {
        variant_name: Ident,
    },
    DeadMethod,
    StructFieldNeverRead,
    ShadowingReservedRegister {
        reg_name: Ident,
    },
    DeadStorageDeclaration,
    DeadStorageDeclarationForFunction {
        unneeded_attrib: String,
    },
    MatchExpressionUnreachableArm,
    UnrecognizedAttribute {
        attrib_name: Ident,
    },
}

impl fmt::Display for Warning {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Warning::*;
        match self {
            NonClassCaseStructName { struct_name } => {
                write!(f,
                "Struct name \"{}\" is not idiomatic. Structs should have a ClassCase name, like \
                 \"{}\".",
                struct_name,
                to_upper_camel_case(struct_name.as_str())
            )
            }
            NonClassCaseTypeParameter { name } => {
                write!(f,
                "Type parameter \"{}\" is not idiomatic. TypeParameters should have a ClassCase name, like \
                 \"{}\".",
                name,
                to_upper_camel_case(name.as_str())
            )
            }
            NonClassCaseTraitName { name } => {
                write!(f,
                "Trait name \"{}\" is not idiomatic. Traits should have a ClassCase name, like \
                 \"{}\".",
                name,
                to_upper_camel_case(name.as_str())
            )
            }
            NonClassCaseEnumName { enum_name } => write!(
                f,
                "Enum \"{}\"'s capitalization is not idiomatic. Enums should have a ClassCase \
                 name, like \"{}\".",
                enum_name,
                to_upper_camel_case(enum_name.as_str())
            ),
            NonSnakeCaseStructFieldName { field_name } => write!(
                f,
                "Struct field name \"{}\" is not idiomatic. Struct field names should have a \
                 snake_case name, like \"{}\".",
                field_name,
                to_snake_case(field_name.as_str())
            ),
            NonClassCaseEnumVariantName { variant_name } => write!(
                f,
                "Enum variant name \"{}\" is not idiomatic. Enum variant names should be \
                 ClassCase, like \"{}\".",
                variant_name,
                to_upper_camel_case(variant_name.as_str())
            ),
            NonSnakeCaseFunctionName { name } => {
                write!(f,
                "Function name \"{}\" is not idiomatic. Function names should be snake_case, like \
                 \"{}\".",
                name,
                to_snake_case(name.as_str())
            )
            }
            NonScreamingSnakeCaseConstName { name } => {
                write!(
                    f,
                    "Constant name \"{}\" is not idiomatic. Constant names should be SCREAMING_SNAKE_CASE, like \
                    \"{}\".",
                    name,
                    to_screaming_snake_case(name.as_str()),
                )
            },
            LossOfPrecision {
                initial_type,
                cast_to,
            } => write!(f,
                "This cast, from integer type of width {} to integer type of width {}, will lose precision.",
                initial_type,
                cast_to
            ),
            UnusedReturnValue { r#type } => write!(
                f,
                "This returns a value of type {}, which is not assigned to anything and is \
                 ignored.",
                r#type
            ),
            SimilarMethodFound { lib, module, name } => write!(
                f,
                "A method with the same name was found for type {} in dependency \"{}::{}\". \
                 Traits must be in scope in order to access their methods. ",
                name, lib, module
            ),
            ShadowsOtherSymbol { name } => write!(
                f,
                "This shadows another symbol in this scope with the same name \"{}\".",
                name
            ),
            OverridingTraitImplementation => write!(
                f,
                "This trait implementation overrides another one that was previously defined."
            ),
            DeadDeclaration => write!(f, "This declaration is never used."),
            DeadStructDeclaration => write!(f, "This struct is never used."),
            DeadFunctionDeclaration => write!(f, "This function is never called."),
            UnreachableCode => write!(f, "This code is unreachable."),
            DeadEnumVariant { variant_name } => {
                write!(f, "Enum variant {} is never constructed.", variant_name)
            }
            DeadTrait => write!(f, "This trait is never implemented."),
            DeadMethod => write!(f, "This method is never called."),
            StructFieldNeverRead => write!(f, "This struct field is never accessed."),
            ShadowingReservedRegister { reg_name } => write!(
                f,
                "This register declaration shadows the reserved register, \"{}\".",
                reg_name
            ),
            DeadStorageDeclaration => write!(
                f,
                "This storage declaration is never accessed and can be removed."
            ),
            DeadStorageDeclarationForFunction { unneeded_attrib } => write!(
                f,
                "The '{unneeded_attrib}' storage declaration for this function is never accessed \
                and can be removed."
            ),
            MatchExpressionUnreachableArm => write!(f, "This match arm is unreachable."),
            UnrecognizedAttribute {attrib_name} => write!(f, "Unknown attribute: \"{attrib_name}\"."),
        }
    }
}
