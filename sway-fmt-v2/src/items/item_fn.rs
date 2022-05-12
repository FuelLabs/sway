use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemFn;

impl Format for ItemFn {
    fn format(&self, _formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
