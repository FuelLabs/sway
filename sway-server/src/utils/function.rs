use core_lang::FunctionDeclaration;

pub(crate) fn extract_fn_signature(func_dec: &FunctionDeclaration) -> String {
    let value = func_dec.span.as_str();
    value.split("{").take(1).map(|v| v.trim()).collect()
}
