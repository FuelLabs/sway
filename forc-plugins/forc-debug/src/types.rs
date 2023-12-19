pub type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
