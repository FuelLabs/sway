use crate::fmt::{Format, FormattedCode, Formatter};
use sway_parse::ItemAbi;
use sway_types::Spanned;

impl Format for ItemAbi {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        self.span().as_str().to_string()
    }
}
