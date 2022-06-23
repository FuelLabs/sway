use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemStruct;

impl FormatItem for ItemStruct {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
