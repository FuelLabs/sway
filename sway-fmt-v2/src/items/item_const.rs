use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemConst;

impl Format for ItemConst {
    fn format(&self, formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
