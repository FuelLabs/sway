use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemUse;
use sway_types::Spanned;

impl Format for ItemUse {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        self.span().as_str().to_string()
    }
}
