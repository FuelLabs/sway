//! Tools related to handling/recovering from Sway compile errors and reporting them to the user.

use sway_types::{Span, Spanned};

use crate::{
    language::parsed::{Expression, VariableDeclaration},
    namespace::ModulePath,
    Engines, GenericArgument, Namespace,
};

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

/// When providing suggestions for errors and warnings, a solution for an issue can sometimes
/// be changing the code in some other module. We want to provide such suggestions only if
/// the programmer can actually change the code in that module.
///
/// Assuming that the issue occurs in the `issue_namespace` to which the programmer has access,
/// and that fixing it means changing the code in the module given by the `absolute_module_path`
/// this function returns true if the programmer can change that module.
pub(crate) fn module_can_be_changed(
    _engines: &Engines,
    issue_namespace: &Namespace,
    absolute_module_path: &ModulePath,
) -> bool {
    // For now, we assume that the programmers can change the module
    // if the module is in the same package where the issue is.
    // A bit too restrictive, considering the same workspace might be more appropriate,
    // but it's a good start.
    !issue_namespace.module_is_external(absolute_module_path)
}

// Returns the span covering all the arguments of a function or intrinsic function call.
// If `arguments` are empty, returns `span`.
pub(crate) fn span_of_arguments(arguments: &[Expression], span: &Span) -> Span {
    if arguments.is_empty() {
        span.clone()
    } else {
        Span::join_all(arguments.iter().map(|arg| arg.span()))
    }
}

// Returns the span covering all the type arguments of a function or intrinsic function call.
// If `type_arguments` are empty, returns `span`.
pub(crate) fn span_of_type_arguments(type_arguments: &[GenericArgument], span: &Span) -> Span {
    if type_arguments.is_empty() {
        span.clone()
    } else {
        Span::join_all(type_arguments.iter().map(|targ| targ.span()))
    }
}
