use sway_types::Span;

use crate::{
    error::{err, ok},
    CompileError, CompileResult,
};

use super::patstack::PatStack;

/// A `Matrix` is a `Vec<PatStack>` that is implemented with special methods
/// particular to the match exhaustivity algorithm.
///
/// The number of rows of the `Matrix` is equal to the number of `PatStack`s and
/// the number of columns of the `Matrix` is equal to the number of elements in
/// the `PatStack`s. Each `PatStack` should contains the same number of
/// elements.
#[derive(Clone, Debug)]
pub(crate) struct Matrix {
    rows: Vec<PatStack>,
}

impl Matrix {
    /// Creates an empty `Matrix`.
    pub(crate) fn empty() -> Self {
        Matrix { rows: vec![] }
    }

    /// Creates a `Matrix` with one row from a `PatStack`.
    pub(crate) fn from_pat_stack(pat_stack: PatStack) -> Self {
        Matrix {
            rows: vec![pat_stack],
        }
    }

    /// Pushes a `PatStack` onto the `Matrix`.
    pub(crate) fn push(&mut self, row: PatStack) {
        self.rows.push(row);
    }

    /// Appends a `Vec<PatStack>` onto the `Matrix`.
    pub(crate) fn append(&mut self, rows: &mut Vec<PatStack>) {
        self.rows.append(rows);
    }

    /// Returns a reference to the rows of the `Matrix`.
    pub(crate) fn rows(&self) -> &Vec<PatStack> {
        &self.rows
    }

    /// Returns the rows of the `Matrix`.
    pub(crate) fn into_rows(self) -> Vec<PatStack> {
        self.rows
    }

    /// Returns the number of rows *m* and the number of columns *n* of the
    /// `Matrix` in the form (*m*, *n*).
    pub(crate) fn m_n(&self, span: &Span) -> CompileResult<(usize, usize)> {
        let warnings = vec![];
        let mut errors = vec![];
        let first = match self.rows.first() {
            Some(first) => first,
            None => return ok((0, 0), warnings, errors),
        };
        let n = first.len();
        for row in self.rows.iter().skip(1) {
            if row.len() != n {
                errors.push(CompileError::Internal(
                    "found invalid matrix size",
                    span.clone(),
                ));
                return err(warnings, errors);
            }
        }
        ok((self.rows.len(), n), warnings, errors)
    }

    /// Reports if the `Matrix` is equivalent to a vector (aka a single
    /// `PatStack`).
    pub(crate) fn is_a_vector(&self) -> bool {
        self.rows.len() == 1
    }

    /// Checks to see if the `Matrix` is a vector, and if it is, returns the
    /// single `PatStack` from its elements.
    pub(crate) fn unwrap_vector(&self, span: &Span) -> CompileResult<PatStack> {
        let warnings = vec![];
        let mut errors = vec![];
        if !self.is_a_vector() {
            errors.push(CompileError::Internal(
                "found invalid matrix size",
                span.clone(),
            ));
            return err(warnings, errors);
        }
        match self.rows.first() {
            Some(first) => ok(first.clone(), warnings, errors),
            None => {
                errors.push(CompileError::Internal(
                    "found invalid matrix size",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Computes Σ, where Σ is a `PatStack` containing the first element of
    /// every row of the `Matrix`.
    pub(crate) fn compute_sigma(&self, span: &Span) -> CompileResult<PatStack> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pat_stack = PatStack::empty();
        for row in self.rows.iter() {
            let first = check!(
                row.first(span),
                return err(warnings, errors),
                warnings,
                errors
            );
            pat_stack.push(first)
        }
        ok(pat_stack.flatten().filter_out_wildcards(), warnings, errors)
    }
}
