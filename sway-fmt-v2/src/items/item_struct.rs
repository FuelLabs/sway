use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemStruct;

impl Format for ItemStruct {
    fn format(&self, _formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
