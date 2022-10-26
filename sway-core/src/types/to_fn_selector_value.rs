use crate::CompileResult;

pub(crate) trait ToFnSelector {
    fn to_fn_selector_value_untruncated(&self) -> CompileResult<Vec<u8>>;

    /// Converts `Self` into a value that is to be used in contract function
    /// selectors.
    /// Hashes the name and parameters using SHA256, and then truncates to four
    /// bytes.
    fn to_fn_selector_value(&self) -> CompileResult<[u8; 4]>;

    fn to_selector_name(&self) -> CompileResult<String>;
}
