use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemEnum;

impl Format for ItemEnum {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
