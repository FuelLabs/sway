use crate::fmt::FormatterError;
pub trait ItemLen {
    fn get_formatted_len(&self) -> Result<usize, FormatterError>;
}
