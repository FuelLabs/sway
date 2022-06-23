use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemAbi;

impl FormatItem for ItemAbi {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
