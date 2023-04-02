/// Takes a formatted function signature & body and returns only the signature.
pub(crate) fn trim_fn_body(f: String) -> String {
    match f.find('{') {
        Some(index) => f.split_at(index).0.to_string(),
        None => f,
    }
}
