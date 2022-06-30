use crate::{
    fmt::{Format, FormattedCode, Formatter},
    FormatterError,
};
use sway_parse::ItemImpl;

impl Format for ItemImpl {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        todo!()
    }
}
