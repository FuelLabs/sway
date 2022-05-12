use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemFn;

impl Format for ItemFn {
    fn format(&self, formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
