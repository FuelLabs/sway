use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemImpl;

impl FormatItem for ItemImpl {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
