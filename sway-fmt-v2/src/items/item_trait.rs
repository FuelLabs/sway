use crate::fmt::{FormatItem, FormattedCode, Formatter};
use sway_parse::ItemTrait;

impl FormatItem for ItemTrait {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
