use crate::{
    fmt::{Format, FormattedCode, Formatter},
    FormatterError,
};
use sway_parse::ItemUse;

impl Format for ItemUse {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        todo!()
    }
}
