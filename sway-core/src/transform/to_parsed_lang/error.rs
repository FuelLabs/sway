use sway_error::{error::CompileError, warning::CompileWarning};

#[derive(Debug, Default)]
/// Contains any errors or warnings that were generated during the conversion into the parse tree.
/// Typically these warnings and errors are populated as a side effect in the `From` and `Into`
/// implementations of error types into [ErrorEmitted].
pub struct ErrorContext {
    pub(crate) warnings: Vec<CompileWarning>,
    pub(crate) errors: Vec<CompileError>,
}

#[derive(Debug)]
/// Represents that an error was emitted to the error context. This struct does not contain the
/// error, rather, other errors are responsible for pushing to the [ErrorContext] in their `Into`
/// implementations.
pub struct ErrorEmitted {
    _priv: (),
}

impl ErrorContext {
    #[allow(dead_code)]
    pub fn warning<W>(&mut self, warning: W)
    where
        W: Into<CompileWarning>,
    {
        self.warnings.push(warning.into());
    }

    pub fn error<E>(&mut self, error: E) -> ErrorEmitted
    where
        E: Into<CompileError>,
    {
        self.errors.push(error.into());
        ErrorEmitted { _priv: () }
    }

    pub fn errors<I, E>(&mut self, errors: I) -> Option<ErrorEmitted>
    where
        I: IntoIterator<Item = E>,
        E: Into<CompileError>,
    {
        let mut emitted_opt = None;
        self.errors.extend(errors.into_iter().map(|error| {
            emitted_opt = Some(ErrorEmitted { _priv: () });
            error.into()
        }));
        emitted_opt
    }
}
