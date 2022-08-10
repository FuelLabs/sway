use std::sync::Arc;
use sway_fmt::{get_formatted_data, FormattingOptions};
use tower_lsp::lsp_types::{Position, Range, TextEdit};

pub fn get_format_text_edits(text: Arc<str>, options: FormattingOptions) -> Option<Vec<TextEdit>> {
    // we only format if code is correct

    match get_formatted_data(text.clone(), options, None) {
        Ok((num_of_lines, formatted_text)) => {
            let text_lines_count = text.split('\n').count();
            let line_end = std::cmp::max(num_of_lines, text_lines_count) as u32;

            let main_edit = TextEdit {
                range: Range::new(Position::new(0, 0), Position::new(line_end as u32, 0)),
                new_text: formatted_text,
            };

            Some(vec![main_edit])
        }
        _ => None,
    }
}
