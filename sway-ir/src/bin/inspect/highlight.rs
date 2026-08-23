//! Basic Sway-IR syntax highlighting for the TUI main panel.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Token styles used by the highlighter.
#[derive(Clone, Copy)]
enum Kind {
    Comment,
    Keyword,
    Opcode,
    Type,
    Number,
    String,
    Metadata,
    Label,
    Ident,
    Punct,
    DiffAdd,
    DiffRemove,
    DiffContext,
    Plain,
}

impl Kind {
    fn style(self) -> Style {
        match self {
            Kind::Comment => Style::default().fg(Color::DarkGray),
            Kind::Keyword => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            Kind::Opcode => Style::default().fg(Color::Cyan),
            Kind::Type => Style::default().fg(Color::Yellow),
            Kind::Number => Style::default().fg(Color::LightGreen),
            Kind::String => Style::default().fg(Color::Green),
            Kind::Metadata => Style::default().fg(Color::LightBlue),
            Kind::Label => Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
            Kind::Ident => Style::default().fg(Color::White),
            Kind::Punct => Style::default().fg(Color::Gray),
            // Very dark backgrounds so syntax colors stay readable.
            Kind::DiffAdd => Style::default()
                .fg(Color::Rgb(160, 230, 160))
                .bg(Color::Rgb(0, 32, 0)),
            Kind::DiffRemove => Style::default()
                .fg(Color::Rgb(230, 160, 160))
                .bg(Color::Rgb(32, 0, 0)),
            Kind::DiffContext => Style::default().fg(Color::DarkGray),
            Kind::Plain => Style::default(),
        }
    }
}

const DIFF_ADD_BG: Color = Color::Rgb(0, 32, 0);
const DIFF_REMOVE_BG: Color = Color::Rgb(32, 0, 0);

const KEYWORDS: &[&str] = &[
    "fn",
    "pub",
    "entry",
    "entry_orig",
    "fallback",
    "local",
    "mut",
    "script",
    "contract",
    "predicate",
    "library",
    "global",
    "configurable",
    "storage_key",
    "true",
    "false",
    "to",
    "key",
    "x",
    "wide",
    "inline",
    "span",
];

const TYPES: &[&str] = &[
    "bool", "u8", "u16", "u32", "u64", "u256", "b256", "unit", "string", "__ptr", "ptr",
];

const OPCODES: &[&str] = &[
    "add",
    "sub",
    "mul",
    "div",
    "and",
    "or",
    "xor",
    "lsh",
    "rsh",
    "mod",
    "not",
    "eq",
    "lt",
    "gt",
    "asm",
    "bitcast",
    "br",
    "call",
    "cast_ptr",
    "cbr",
    "cmp",
    "const",
    "contract_call",
    "get_elem_ptr",
    "get_local",
    "get_global",
    "get_config",
    "get_storage_key",
    "gtf",
    "int_to_ptr",
    "alloc",
    "load",
    "log",
    "mem_copy_bytes",
    "mem_copy_val",
    "mem_clear_val",
    "nop",
    "ptr_to_int",
    "read_register",
    "ret",
    "retd",
    "revert",
    "jmp_mem",
    "smo",
    "state_clear",
    "state_clear_slots",
    "state_load_quad_word",
    "state_read_slot",
    "state_load_word",
    "state_store_quad_word",
    "state_write_slot",
    "state_update_slot",
    "state_preload",
    "state_store_word",
    "store",
    "init_aggr",
];

/// Highlight a full IR (or diff) document into styled lines.
pub(crate) fn highlight_ir(text: &str, is_diff: bool) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| highlight_line(line, is_diff))
        .collect()
}

fn highlight_line(line: &str, is_diff: bool) -> Line<'static> {
    // Diff prefixes from prettydiff-style output.
    let (prefix, rest, diff_kind) = if is_diff {
        if let Some(r) = line.strip_prefix("+ ") {
            ("+ ", r, Some(Kind::DiffAdd))
        } else if let Some(r) = line.strip_prefix("- ") {
            ("- ", r, Some(Kind::DiffRemove))
        } else if let Some(r) = line.strip_prefix("  ") {
            ("  ", r, Some(Kind::DiffContext))
        } else {
            ("", line, None)
        }
    } else {
        ("", line, None)
    };

    // Full-line comments.
    let trimmed = rest.trim_start();
    let built = if trimmed.starts_with("//") {
        let mut spans = Vec::new();
        if !prefix.is_empty() {
            spans.push(Span::styled(
                prefix.to_string(),
                diff_kind.unwrap_or(Kind::Plain).style(),
            ));
        }
        spans.push(Span::styled(rest.to_string(), Kind::Comment.style()));
        Line::from(spans)
    } else if looks_like_label(trimmed) {
        // Block labels: `name():` or `name:`
        let mut spans = Vec::new();
        if !prefix.is_empty() {
            spans.push(Span::styled(
                prefix.to_string(),
                diff_kind.unwrap_or(Kind::Plain).style(),
            ));
        }
        let lead = rest.len() - trimmed.len();
        if lead > 0 {
            spans.push(Span::raw(rest[..lead].to_string()));
        }
        spans.push(Span::styled(trimmed.to_string(), Kind::Label.style()));
        Line::from(spans)
    } else {
        let mut spans = Vec::new();
        if !prefix.is_empty() {
            spans.push(Span::styled(
                prefix.to_string(),
                diff_kind.unwrap_or(Kind::Plain).style(),
            ));
        }
        spans.extend(tokenize(rest));
        Line::from(spans)
    };

    apply_diff_background(built, diff_kind)
}

/// Paint the whole line with a very dark red/green background for diffs.
fn apply_diff_background(line: Line<'static>, diff_kind: Option<Kind>) -> Line<'static> {
    let bg = match diff_kind {
        Some(Kind::DiffAdd) => DIFF_ADD_BG,
        Some(Kind::DiffRemove) => DIFF_REMOVE_BG,
        _ => return line,
    };
    let spans: Vec<Span<'static>> = line
        .spans
        .into_iter()
        .map(|s| Span::styled(s.content.to_string(), s.style.bg(bg)))
        .collect();
    Line::from(spans).style(Style::default().bg(bg))
}

fn looks_like_label(trimmed: &str) -> bool {
    if !trimmed.ends_with(':') {
        return false;
    }
    let name = trimmed.trim_end_matches(':').trim_end_matches("()");
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn tokenize(s: &str) -> Vec<Span<'static>> {
    let bytes = s.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // Keep `i` on a char boundary at the top of every iteration.
        debug_assert!(s.is_char_boundary(i));

        // Whitespace (ASCII only; Unicode whitespace is emitted below).
        if bytes[i].is_ascii_whitespace() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            spans.push(Span::raw(s[start..i].to_string()));
            continue;
        }

        // Line comment
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            spans.push(Span::styled(s[i..].to_string(), Kind::Comment.style()));
            break;
        }

        // String literal
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    // Skip the backslash and the next Unicode scalar. Using
                    // `i + 2` can land mid-UTF-8 and panic on a later slice.
                    i += 1;
                    if i < bytes.len() {
                        i = next_char_boundary(s, i);
                    }
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i = next_char_boundary(s, i);
            }
            spans.push(Span::styled(s[start..i].to_string(), Kind::String.style()));
            continue;
        }

        // Metadata `!123`
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            spans.push(Span::styled(
                s[start..i].to_string(),
                Kind::Metadata.style(),
            ));
            continue;
        }

        // Number (decimal or hex)
        if bytes[i].is_ascii_digit() {
            let start = i;
            if i + 1 < bytes.len()
                && bytes[i] == b'0'
                && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X')
            {
                i += 2;
                while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                    i += 1;
                }
            } else {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            spans.push(Span::styled(s[start..i].to_string(), Kind::Number.style()));
            continue;
        }

        // Identifier / keyword / opcode / type
        if is_ident_start(bytes[i]) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_cont(bytes[i]) {
                i += 1;
            }
            let word = &s[start..i];
            let kind = classify_word(word);
            spans.push(Span::styled(word.to_string(), kind.style()));
            continue;
        }

        // One Unicode scalar (ASCII punct or any non-ASCII). Never slice a
        // single byte out of a multi-byte UTF-8 character — that panics.
        let next = next_char_boundary(s, i);
        spans.push(Span::styled(
            s[i..next].to_string(),
            Kind::Punct.style(),
        ));
        i = next;
    }

    spans
}

/// Advance from a char boundary to the start of the following character.
fn next_char_boundary(s: &str, i: usize) -> usize {
    s[i..]
        .chars()
        .next()
        .map(|c| i + c.len_utf8())
        .unwrap_or(s.len())
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn classify_word(word: &str) -> Kind {
    if KEYWORDS.contains(&word) {
        Kind::Keyword
    } else if OPCODES.contains(&word) {
        Kind::Opcode
    } else if TYPES.contains(&word) {
        Kind::Type
    } else {
        Kind::Ident
    }
}

/// Highlight with an optional search match overlay (case-insensitive).
pub(crate) fn highlight_ir_with_search(
    text: &str,
    is_diff: bool,
    search: &str,
    current_match_line: Option<usize>,
) -> Vec<Line<'static>> {
    if search.is_empty() {
        return highlight_ir(text, is_diff);
    }
    let needle = search.to_ascii_lowercase();
    text.lines()
        .enumerate()
        .map(|(idx, line)| {
            let mut styled = highlight_line(line, is_diff);
            let lower = line.to_ascii_lowercase();
            if lower.contains(&needle) {
                let mark = if Some(idx) == current_match_line {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::Rgb(60, 60, 20))
                };
                // Re-wrap with a subtle background on the whole line for matches.
                let content: Vec<Span> = styled
                    .spans
                    .drain(..)
                    .map(|s| {
                        let mut st = s.style;
                        if let Some(bg) = mark.bg {
                            st = st.bg(bg);
                        }
                        if let Some(fg) = mark.fg {
                            if Some(idx) == current_match_line {
                                st = st.fg(fg);
                            }
                        }
                        Span::styled(s.content.to_string(), st)
                    })
                    .collect();
                styled = Line::from(content);
            }
            styled
        })
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_in_comment_does_not_panic() {
        let _ = highlight_ir("// comment → … ● 日本語\n", false);
    }

    #[test]
    fn unicode_in_code_does_not_panic() {
        let _ = highlight_ir("v0 = const u64 1 // →\nfn 日本語() {\n}\n", false);
    }

    #[test]
    fn unicode_string_escape_does_not_panic() {
        // Backslash followed by a multi-byte char previously skipped mid-char.
        let _ = highlight_ir("v0 = const string \"a\\→b\"\n", false);
    }

    #[test]
    fn diff_with_unicode_does_not_panic() {
        let _ = highlight_ir("+   // added →\n-   // removed …\n  // context ●\n", true);
    }
}


#[cfg(test)]
mod version_toggle_tests {
    use super::*;
    use crate::parse::{strip_version_suffix, strip_metadata_and_version, MdMode, VersionMode};

    #[test]
    fn highlighting_after_version_strip_does_not_panic() {
        let sample = r#"
script {
    fn main() -> () {
        entry_block():
        v0v0 = const u64 1, !1
        v1v2 = add v0v0, v0v0, !2
        ret ()
    }
}
!1 = span 0 1 2
"#;
        let stripped = strip_version_suffix(sample);
        let _ = highlight_ir(&stripped, false);
        let both = strip_metadata_and_version(sample, MdMode::Without, VersionMode::Without);
        let _ = highlight_ir(&both, false);
        let _ = highlight_ir_with_search(&stripped, false, "v0", Some(0));
    }

    #[test]
    fn highlighting_real_ir_after_version_strip() {
        use std::path::PathBuf;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        if !root.exists() { return; }
        for entry in std::fs::read_dir(&root).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("ir") { continue; }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let stripped = strip_version_suffix(&text);
            let _ = highlight_ir(&stripped, false);
            let _ = highlight_ir(&stripped, true);
        }
    }
}
