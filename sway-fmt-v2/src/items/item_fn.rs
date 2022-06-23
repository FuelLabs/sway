use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemFn;

impl FormatItem for ItemFn {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
