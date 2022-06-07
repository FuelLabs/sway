use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemUse;

impl Format for ItemUse {
    fn format(&self, _formatter: &Formatter) -> FormattedCode {
        todo!()
        // reorder use statements alphabeticaly
        // break into new lines for long use statements
    }
}
