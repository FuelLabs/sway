use crate::{config::items::ItemBraceStyle, fmt::VirtualItemKind, Formatter};

/// Handles bracket open scenerio. Checks the config for the placement of the bracket.
/// Modifies the current shape of the formatter.
pub fn handle_open_bracket(
    push_to: &mut String,
    formatter: &mut Formatter,
    item_kind: VirtualItemKind,
) {
    let bracket_on_new_line = formatter.config.items.item_brace_style;
    let mut shape = formatter.shape;
    match item_kind {
        VirtualItemKind::Use => todo!(),
        VirtualItemKind::Struct => todo!(),
        VirtualItemKind::Enum => {
            match bracket_on_new_line {
                ItemBraceStyle::AlwaysNextLine => {
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
        }
        VirtualItemKind::Fn => todo!(),
        VirtualItemKind::Trait => todo!(),
        VirtualItemKind::Impl => todo!(),
        VirtualItemKind::Abi => todo!(),
        VirtualItemKind::Const => todo!(),
        VirtualItemKind::Storage => todo!(),
    };

    formatter.shape = shape;
}

/// Handles bracket close scenerio.
/// Currently it simply pushes a `}` and modifies the shape.
pub fn handle_close_bracket(push_to: &mut String, formatter: &mut Formatter) {
    push_to.push('}');
    // If shape is becoming left-most alligned or - indent just have the defualt shape
    formatter.shape = formatter.shape.shrink_left(1).unwrap_or_default();
}
