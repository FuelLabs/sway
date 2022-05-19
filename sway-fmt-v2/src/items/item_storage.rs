use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemStorage;

impl Format for ItemStorage {
    fn format(&self, _formatter: &Formatter) -> FormattedCode {
        todo!()
    }
}
