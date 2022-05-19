use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemAbi;

impl Format for ItemAbi {
    fn format(&self, _formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
