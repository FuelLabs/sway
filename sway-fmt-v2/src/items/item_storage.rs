use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemStorage;

impl FormatItem for ItemStorage {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
