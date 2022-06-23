use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemUse;

impl FormatItem for ItemUse {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
