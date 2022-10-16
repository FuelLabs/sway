//! Tools related to handling/recovering from Sway compile errors and reporting them to the user.

use crate::language::parsed::VariableDeclaration;
use sway_error::error::CompileError;
use sway_error::warning::CompileWarning;

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
            use sway_error::warning::CompileWarning;
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
