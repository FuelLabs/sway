use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemTrait;

impl Format for ItemTrait {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}
