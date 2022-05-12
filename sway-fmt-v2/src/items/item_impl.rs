use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemImpl;

impl Format for ItemImpl {
    fn format(&self, formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
