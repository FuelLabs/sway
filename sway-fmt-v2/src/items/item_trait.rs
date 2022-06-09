use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::indent_style::Shape,
};
use sway_parse::ItemTrait;

impl Format for ItemTrait {
    fn format(&self, _formatter: &Formatter, _shape: &mut Shape) -> FormattedCode {
        todo!()
    }
}
