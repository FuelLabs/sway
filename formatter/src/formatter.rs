use super::code_builder::CodeBuilder;

/// returns number of lines and formatted text
pub fn get_formatted_data(file: &str, tab_size: u32) -> (usize, String) {
    let mut code_builder = CodeBuilder::new(tab_size);
    let lines: Vec<&str> = file.split("\n").collect();

    // todo: handle lengthy lines of code
    for line in lines {
        code_builder.format_and_add(line);
    }

    code_builder.get_final_edits()
}
