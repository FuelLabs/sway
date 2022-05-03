use sway_types::Span;

/// Expects a span from either a FunctionDeclaration or a TypedFunctionDeclaration
pub(crate) fn extract_fn_signature(span: &Span) -> String {
    let value = span.as_str();
    value.split('{').take(1).map(|v| v.trim()).collect()
}
