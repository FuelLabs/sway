use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemConst;

impl Format for ItemConst {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
