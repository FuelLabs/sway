library;

pub fn panic_keyword_behind_feature_flag_with_expression() {
    panic "This is a panic string in a panic expression.";
}

pub fn panic_keyword_behind_feature_flag_without_expression() {
    panic;
}

pub fn panic_keyword_behind_feature_flag_without_expression_without_semicolon() {
    panic
}
