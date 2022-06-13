use crate::Formatter;
use sway_parse::Ty;
use sway_types::Spanned;

/// Handling of a ty, currently this does not apply any formatting.
/// It simply pushes the string version of the ty.
pub fn handle_ty(ty: &Ty, push_to: &mut String) {
    push_to.push_str(ty.span().as_str());
}

/// Handles bracket open scenerio. Checks the config for the placement of the bracket.
/// Modifies the current shape of the formatter.
pub fn handle_bracket_open(push_to: &mut String, formatter: &mut Formatter) {
    let bracket_on_new_line = formatter.config.items.item_brace_style;
    let mut shape = formatter.shape;
    match bracket_on_new_line {
        crate::config::items::ItemBraceStyle::AlwaysNextLine => {
            // Add openning bracet to the next line.
            push_to.push_str("\n{\n");
            shape = shape.block_indent(1);
        }
        _ => {
            // Add opening bracet to the same line
            push_to.push_str(" {\n");
            shape = shape.block_indent(1);
        }
    }
    formatter.shape = shape;
}

/// Handles bracket close scenerio.
/// Currently it simply pushes a `}` and modifies the shape.
pub fn handle_bracket_close(push_to: &mut String, formatter: &mut Formatter) {
    push_to.push('}');
    // If shape is becoming left-most alligned or - indent just have the defualt shape
    formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
}
