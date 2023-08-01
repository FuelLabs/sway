use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::CompileError;

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
    pub(crate) fn m_n(
        &self,
        handler: &Handler,
        span: &Span,
    ) -> Result<(usize, usize), ErrorEmitted> {
        let first = match self.rows.first() {
            Some(first) => first,
            None => return Ok((0, 0)),
        };
        let n = first.len();
        for row in self.rows.iter().skip(1) {
            if row.len() != n {
                return Err(handler.emit_err(CompileError::Internal(
                    "found invalid matrix size",
                    span.clone(),
                )));
            }
        }
        Ok((self.rows.len(), n))
    }

    /// Computes Σ, where Σ is a `PatStack` containing the first element of
    /// every row of the `Matrix`.
    pub(crate) fn compute_sigma(
        &self,
        handler: &Handler,
        span: &Span,
    ) -> Result<PatStack, ErrorEmitted> {
        let mut pat_stack = PatStack::empty();
        for row in self.rows.iter() {
            let first = row.first(handler, span)?;
            pat_stack.push(first.into_root_constructor())
        }
        Ok(pat_stack.remove_duplicates())
    }
}
