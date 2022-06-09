use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::indent_style::Shape,
};
use sway_parse::ItemStorage;

impl Format for ItemStorage {
    fn format(&self, _formatter: &Formatter, _shape: &mut Shape) -> FormattedCode {
        todo!()
    }
}
