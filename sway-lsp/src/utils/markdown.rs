//! Transforms markdown
const SWAYDOC_FENCES: [&str; 2] = ["```", "~~~"];

/// Transforms markdown and takes care of any code blocks
/// to allow for syntax highlighting.
pub fn format_docs(src: &str) -> String {
    let mut processed_lines = Vec::new();
    let mut in_code_block = false;
    let mut is_sway = false;

    for mut line in src.lines() {
        if in_code_block && is_sway && code_line_ignored_by_swaydoc(line) {
            continue;
        }

        if let Some(header) = SWAYDOC_FENCES
            .into_iter()
            .find_map(|fence| line.strip_prefix(fence))
        {
            in_code_block ^= true;

            if in_code_block {
                is_sway = is_sway_fence(header);

                if is_sway {
                    line = "```sway";
                }
            }
        }

        if in_code_block {
            let trimmed = line.trim_start();
            if trimmed.starts_with("##") {
                line = &trimmed[1..];
            }
        }

        processed_lines.push(line);
    }
    processed_lines.join("\n")
}

fn code_line_ignored_by_swaydoc(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "#" || trimmed.starts_with("# ") || trimmed.starts_with("#\t")
}

// stripped down version of https://github.com/rust-lang/rust/blob/392ba2ba1a7d6c542d2459fb8133bebf62a4a423/src/librustdoc/html/markdown.rs#L810-L933
fn is_sway_fence(s: &str) -> bool {
    let mut seen_sway_tags = false;
    let mut seen_other_tags = false;

    let tokens = s
        .trim()
        .split(|c| matches!(c, ',' | ' ' | '\t'))
        .map(str::trim)
        .filter(|t| !t.is_empty());

    for token in tokens {
        match token {
            "should_panic" | "no_run" | "ignore" | "allow_fail" => {
                seen_sway_tags = !seen_other_tags
            }
            "sway" => seen_sway_tags = true,
            "test_harness" | "compile_fail" => seen_sway_tags = !seen_other_tags || seen_sway_tags,
            _ => seen_other_tags = true,
        }
    }

    !seen_other_tags || seen_sway_tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_docs_adds_sway() {
        let comment = "```\nfn some_sway() {}\n```";
        assert_eq!(format_docs(comment), "```sway\nfn some_sway() {}\n```");
    }

    #[test]
    fn test_format_docs_handles_plain_text() {
        let comment = "```text\nthis is plain text\n```";
        assert_eq!(format_docs(comment), "```text\nthis is plain text\n```");
    }

    #[test]
    fn test_format_docs_handles_non_sway() {
        let comment = "```sh\nsupposedly shell code\n```";
        assert_eq!(format_docs(comment), "```sh\nsupposedly shell code\n```");
    }

    #[test]
    fn test_format_docs_handles_sway_alias() {
        let comment = "```ignore\nlet z = 55;\n```";
        assert_eq!(format_docs(comment), "```sway\nlet z = 55;\n```");
    }

    #[test]
    fn test_format_docs_handles_complex_code_block_attrs() {
        let comment = "```sway,no_run\nlet z = 55;\n```";
        assert_eq!(format_docs(comment), "```sway\nlet z = 55;\n```");
    }

    #[test]
    fn test_format_docs_skips_comments_in_sway_block() {
        let comment =
            "```sway\n # skip1\n# skip2\n#stay1\nstay2\n#\n #\n   #    \n #\tskip3\n\t#\t\n```";
        assert_eq!(format_docs(comment), "```sway\n#stay1\nstay2\n```");
    }

    #[test]
    fn test_format_docs_does_not_skip_lines_if_plain_text() {
        let comment =
            "```text\n # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t\n```";
        assert_eq!(
            format_docs(comment),
            "```text\n # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t\n```",
        );
    }

    #[test]
    fn test_format_docs_keeps_comments_outside_of_sway_block() {
        let comment = " # stay1\n# stay2\n#stay3\nstay4\n#\n #\n   #    \n #\tstay5\n\t#\t";
        assert_eq!(format_docs(comment), comment);
    }

    #[test]
    fn test_format_docs_preserves_newlines() {
        let comment = "this\nis\nmultiline";
        assert_eq!(format_docs(comment), comment);
    }

    #[test]
    fn test_code_blocks_in_comments_marked_as_sway() {
        let comment = r#"```sway
fn main(){}
```
Some comment.
```
let a = 1;
```"#;

        assert_eq!(
            format_docs(comment),
            "```sway\nfn main(){}\n```\nSome comment.\n```sway\nlet a = 1;\n```"
        );
    }

    #[test]
    fn test_code_blocks_in_comments_marked_as_text() {
        let comment = r#"```text
filler
text
```
Some comment.
```
let a = 1;
```"#;

        assert_eq!(
            format_docs(comment),
            "```text\nfiller\ntext\n```\nSome comment.\n```sway\nlet a = 1;\n```"
        );
    }

    #[test]
    fn test_format_docs_handles_escape_double_hashes() {
        let comment = r#"```sway
let s = "foo
## bar # baz";
```"#;

        assert_eq!(
            format_docs(comment),
            "```sway\nlet s = \"foo\n# bar # baz\";\n```"
        );
    }
}
