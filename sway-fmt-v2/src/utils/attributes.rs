use crate::Formatter;
use sway_parse::AttributeDecl;
use sway_types::Spanned;
pub fn format_attributes(attributes: Vec<AttributeDecl>, _formatter: &mut Formatter) -> String {
    // TODO format attributes
    attributes
        .into_iter()
        .map(|x| x.span().as_str().to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
