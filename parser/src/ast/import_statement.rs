#[derive(Debug)]
pub(crate) struct ImportStatement<'sc> {
    root: &'sc str,
    path: Vec<&'sc str>,
}
