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
    /// Embedded Sway source lines (`  | …`) from the source overlay.
    Source,
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
            Kind::Source => Style::default().fg(Color::Rgb(120, 200, 140)),
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

    // Full-line comments / embedded source lines from the source overlay.
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
    } else if rest.starts_with("  |") || trimmed.starts_with("| ") {
        // `  | <source>` lines inserted by the source overlay.
        let mut spans = Vec::new();
        if !prefix.is_empty() {
            spans.push(Span::styled(
                prefix.to_string(),
                diff_kind.unwrap_or(Kind::Plain).style(),
            ));
        }
        spans.push(Span::styled(rest.to_string(), Kind::Source.style()));
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

/// Background palette for multi-SSA focus highlights (overrides syntax bg).
/// Shared palette for SSA / local / method focus highlights.
pub(crate) const SYMBOL_PALETTE: &[Color] = &[
    Color::Rgb(120, 70, 0),   // amber
    Color::Rgb(0, 90, 110),   // teal
    Color::Rgb(110, 0, 110),  // purple
    Color::Rgb(0, 100, 40),   // green
    Color::Rgb(130, 30, 30),  // red
    Color::Rgb(30, 50, 140),  // blue
    Color::Rgb(110, 100, 0),  // olive
    Color::Rgb(130, 50, 80),  // rose
];

/// Back-compat alias.
pub(crate) const SSA_PALETTE: &[Color] = SYMBOL_PALETTE;

/// True for Sway-IR value names like `v0`, `v12`, `v3v1`.
pub(crate) fn is_ssa_name(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() < 2 || b[0] != b'v' {
        return false;
    }
    let mut i = 1;
    if i >= b.len() || !b[i].is_ascii_digit() {
        return false;
    }
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == b.len() {
        return true;
    }
    if b[i] != b'v' {
        return false;
    }
    i += 1;
    if i >= b.len() || !b[i].is_ascii_digit() {
        return false;
    }
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i == b.len()
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// SSA register covering character column `col` (0-based), if any.
pub(crate) fn ssa_at_col(line: &str, col: usize) -> Option<String> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    if chars.is_empty() || col >= chars.len() {
        return None;
    }
    let byte_pos = chars[col].0;
    let bytes = line.as_bytes();
    if !is_ident_byte(bytes[byte_pos]) {
        return None;
    }
    let mut start = byte_pos;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = byte_pos + chars[col].1.len_utf8();
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    let tok = line.get(start..end)?;
    is_ssa_name(tok).then(|| tok.to_string())
}


/// Identifier token covering character column `col` (0-based), if any.
pub(crate) fn ident_at_col(line: &str, col: usize) -> Option<String> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    if chars.is_empty() || col >= chars.len() {
        return None;
    }
    let byte_pos = chars[col].0;
    let bytes = line.as_bytes();
    if !is_ident_byte(bytes[byte_pos]) {
        return None;
    }
    let mut start = byte_pos;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = byte_pos + chars[col].1.len_utf8();
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    line.get(start..end).map(str::to_string)
}

fn is_ident_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

/// Skip one IR type (`u64`, `__ptr T`, `{…}`, `[…; N]`) and return the remainder.
fn skip_ir_type(s: &str) -> Option<&str> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('{') {
        return skip_balanced(s, b'{', b'}');
    }
    if s.starts_with('[') {
        return skip_balanced(s, b'[', b']');
    }
    // Leading ident (e.g. u64, __ptr, b256).
    let bytes = s.as_bytes();
    let mut i = 0usize;
    if !is_ident_byte(bytes[0]) {
        return None;
    }
    while i < bytes.len() && is_ident_byte(bytes[i]) {
        i += 1;
    }
    let head = &s[..i];
    let rest = s[i..].trim_start();
    if head == "__ptr" || head == "ptr" {
        return skip_ir_type(rest);
    }
    Some(rest)
}

fn skip_balanced<'a>(s: &'a str, open: u8, close: u8) -> Option<&'a str> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&open) {
        return None;
    }
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Some(s[i + 1..].trim_start());
            }
        }
    }
    None
}

/// Name declared by a `local [mut] <ty> <name>` line.
pub(crate) fn local_decl_name(line: &str) -> Option<&str> {
    let (body, _) = line_instruction_body(line);
    let rest = body.strip_prefix("local")?.trim_start();
    let rest = match rest.strip_prefix("mut") {
        Some(r) => r.trim_start(),
        None => rest,
    };
    let after_ty = skip_ir_type(rest)?;
    let name = after_ty
        .split(|c: char| c == ' ' || c == '=' || c == ',' || c == '!')
        .next()
        .unwrap_or("");
    if is_ident_name(name) && !is_ssa_name(name) {
        Some(name)
    } else {
        None
    }
}

/// Local name after the comma in `get_local …, name`.
pub(crate) fn get_local_name(line: &str) -> Option<&str> {
    let (body, _) = line_instruction_body(line);
    // Match both `get_local` opcode forms, including after `= `.
    let idx = body.find("get_local")?;
    let after = &body[idx + "get_local".len()..];
    let after_comma = after.rsplit_once(',')?.1.trim();
    let name = after_comma
        .split(|c: char| c == ' ' || c == '!' || c == ',')
        .next()
        .unwrap_or("");
    if is_ident_name(name) && !is_ssa_name(name) {
        Some(name)
    } else {
        None
    }
}

/// Function name from `fn name(` / `pub entry fn name(`.
pub(crate) fn fn_def_name(line: &str) -> Option<&str> {
    let (body, _) = line_instruction_body(line);
    let idx = body.find("fn ")?;
    let after = body[idx + 3..].trim_start();
    let name = after.split(|c: char| c == '(' || c == ' ' || c == '!').next()?;
    if is_ident_name(name) && !is_ssa_name(name) {
        Some(name)
    } else {
        None
    }
}

/// Callee from `call name(` / `= call name(`.
pub(crate) fn call_name(line: &str) -> Option<&str> {
    let (body, _) = line_instruction_body(line);
    let idx = body.find("call ")?;
    let after = body[idx + 5..].trim_start();
    let name = after.split(|c: char| c == '(' || c == ' ' || c == '!' || c == ',').next()?;
    if is_ident_name(name) && !is_ssa_name(name) {
        Some(name)
    } else {
        None
    }
}

pub(crate) fn text_has_local(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        local_decl_name(line) == Some(name) || get_local_name(line) == Some(name)
    })
}

pub(crate) fn text_has_method(text: &str, name: &str) -> bool {
    text.lines().any(|line| fn_def_name(line) == Some(name) || call_name(line) == Some(name))
}

/// Highlightable symbol under the cursor: SSA register, local, or method/function name.
pub(crate) fn symbol_at_col(line: &str, col: usize, full_text: &str) -> Option<String> {
    let tok = ident_at_col(line, col)?;
    if is_ssa_name(&tok) {
        return Some(tok);
    }
    if !is_ident_name(&tok) {
        return None;
    }
    // Prefer context on this line, then dump-wide recognition.
    if local_decl_name(line) == Some(tok.as_str())
        || get_local_name(line) == Some(tok.as_str())
        || fn_def_name(line) == Some(tok.as_str())
        || call_name(line) == Some(tok.as_str())
        || text_has_local(full_text, &tok)
        || text_has_method(full_text, &tok)
    {
        return Some(tok);
    }
    None
}

/// Count whole-token occurrences of `name` above `view_start` / at-or-after `view_end`.
pub(crate) fn count_token_offscreen(
    text: &str,
    name: &str,
    view_start: usize,
    view_end: usize,
) -> (usize, usize) {
    if name.is_empty() {
        return (0, 0);
    }
    let name_bytes = name.as_bytes();
    let mut above = 0usize;
    let mut below = 0usize;
    for (line_idx, line) in text.lines().enumerate() {
        let bytes = line.as_bytes();
        let mut start = 0usize;
        let mut n = 0usize;
        while start + name_bytes.len() <= bytes.len() {
            if bytes[start..].starts_with(name_bytes)
                && (start == 0 || !is_ident_byte(bytes[start - 1]))
                && {
                    let end = start + name_bytes.len();
                    end == bytes.len() || !is_ident_byte(bytes[end])
                }
            {
                n += 1;
                start += name_bytes.len();
            } else {
                start += 1;
            }
        }
        if line_idx < view_start {
            above += n;
        } else if line_idx >= view_end {
            below += n;
        }
    }
    (above, below)
}

/// Highlight IR with optional syntax coloring and search overlay.
/// SSA focus highlights are applied separately via [`apply_ssa_highlights`].
pub(crate) fn highlight_ir_with_search(
    text: &str,
    is_diff: bool,
    search: &str,
    current_match_line: Option<usize>,
    syntax: bool,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = if syntax {
        highlight_ir(text, is_diff)
    } else if is_diff {
        text.lines()
            .map(|line| {
                let style = if line.starts_with("+ ") {
                    Kind::DiffAdd.style()
                } else if line.starts_with("- ") {
                    Kind::DiffRemove.style()
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(line.to_string(), style))
            })
            .collect()
    } else {
        text.lines()
            .map(|line| {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::White),
                ))
            })
            .collect()
    };

    if search.is_empty() {
        return lines;
    }

    let needle = search.to_ascii_lowercase();
    for (idx, styled) in lines.iter_mut().enumerate() {
        let line_text: String = styled.spans.iter().map(|s| s.content.as_ref()).collect();
        if !line_text.to_ascii_lowercase().contains(&needle) {
            continue;
        }
        let mark = if Some(idx) == current_match_line {
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::Rgb(60, 60, 20))
        };
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
        *styled = Line::from(content);
    }
    lines
}

/// Paint SSA token backgrounds. Always overrides whatever bg syntax/search set.
pub(crate) fn apply_symbol_highlights(lines: &mut [Line<'static>], highlights: &[(String, Color)]) {
    apply_ssa_highlights(lines, highlights);
}

pub(crate) fn apply_ssa_highlights(lines: &mut [Line<'static>], highlights: &[(String, Color)]) {
    if highlights.is_empty() {
        return;
    }
    for line in lines {
        apply_ssa_to_line(line, highlights);
    }
}

fn apply_ssa_to_line(line: &mut Line<'static>, highlights: &[(String, Color)]) {
    let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    if text.is_empty() {
        return;
    }

    let mut styles: Vec<Style> = Vec::new();
    for span in &line.spans {
        for _ in span.content.chars() {
            styles.push(span.style);
        }
    }
    if styles.len() != text.chars().count() {
        return;
    }

    let chars: Vec<char> = text.chars().collect();
    let bytes = text.as_bytes();
    let mut char_of_byte = vec![0usize; bytes.len() + 1];
    {
        let mut ci = 0usize;
        for (bi, _ch) in text.char_indices() {
            char_of_byte[bi] = ci;
            ci += 1;
        }
        char_of_byte[bytes.len()] = ci;
    }

    for (name, bg) in highlights {
        let name_bytes = name.as_bytes();
        let mut start = 0usize;
        while start + name_bytes.len() <= bytes.len() {
            if bytes[start..].starts_with(name_bytes)
                && (start == 0 || !is_ident_byte(bytes[start - 1]))
                && {
                    let end = start + name_bytes.len();
                    end == bytes.len() || !is_ident_byte(bytes[end])
                }
            {
                let c0 = char_of_byte[start];
                let c1 = char_of_byte[start + name_bytes.len()];
                for style in styles.iter_mut().take(c1).skip(c0) {
                    *style = style.bg(*bg).fg(Color::White);
                }
                start += name_bytes.len();
            } else {
                start += 1;
            }
        }
    }

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let st = styles[i];
        let mut j = i + 1;
        while j < chars.len() && styles[j] == st {
            j += 1;
        }
        out.push(Span::styled(chars[i..j].iter().collect::<String>(), st));
        i = j;
    }
    *line = Line::from(out);
}

/// Drop the first `skip` characters from styled spans, preserving styles.
pub(crate) fn skip_line_chars(line: &Line<'static>, skip: usize) -> Line<'static> {
    if skip == 0 {
        return line.clone();
    }
    let mut remaining = skip;
    let mut out = Vec::new();
    for span in &line.spans {
        if remaining == 0 {
            out.push(span.clone());
            continue;
        }
        let len = span.content.chars().count();
        if remaining >= len {
            remaining -= len;
            continue;
        }
        let kept: String = span.content.chars().skip(remaining).collect();
        remaining = 0;
        out.push(Span::styled(kept, span.style));
    }
    Line::from(out)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SsaRefKind {
    Def,
    Use,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SsaRef {
    pub line: usize,
    pub kind: SsaRefKind,
}

fn strip_diff_prefix(line: &str) -> &str {
    if let Some(r) = line.strip_prefix("+ ") {
        r
    } else if let Some(r) = line.strip_prefix("- ") {
        r
    } else if let Some(r) = line.strip_prefix("  ") {
        r
    } else {
        line
    }
}

/// Instruction text after diff prefix + indent, and its byte offset in `line`.
fn line_instruction_body(line: &str) -> (&str, usize) {
    let after_diff = strip_diff_prefix(line);
    let diff_len = line.len() - after_diff.len();
    let trimmed = after_diff.trim_start();
    let ws = after_diff.len() - trimmed.len();
    (trimmed, diff_len + ws)
}

/// All whole-token defs/uses of `name` in `text` (0-based line indices).
pub(crate) fn find_ssa_refs(text: &str, name: &str) -> Vec<SsaRef> {
    if name.is_empty() || !is_ssa_name(name) {
        return Vec::new();
    }
    let name_bytes = name.as_bytes();
    let mut out = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let (body, body_byte_offset) = line_instruction_body(line);
        let def_here = body.starts_with(name) && body[name.len()..].starts_with(" = ");

        let bytes = line.as_bytes();
        let mut start = 0usize;
        let mut saw_def = false;
        while start + name_bytes.len() <= bytes.len() {
            if bytes[start..].starts_with(name_bytes)
                && (start == 0 || !is_ident_byte(bytes[start - 1]))
                && {
                    let end = start + name_bytes.len();
                    end == bytes.len() || !is_ident_byte(bytes[end])
                }
            {
                let is_def = def_here && !saw_def && start == body_byte_offset;
                if is_def {
                    saw_def = true;
                }
                out.push(SsaRef {
                    line: line_idx,
                    kind: if is_def {
                        SsaRefKind::Def
                    } else {
                        SsaRefKind::Use
                    },
                });
                start += name_bytes.len();
            } else {
                start += 1;
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SsaOffscreen {
    pub above_defs: usize,
    pub above_uses: usize,
    pub below_defs: usize,
    pub below_uses: usize,
}

/// Count defs/uses of `name` strictly above `view_start` or at/after `view_end`.
pub(crate) fn count_ssa_offscreen(
    text: &str,
    name: &str,
    view_start: usize,
    view_end: usize,
) -> SsaOffscreen {
    let mut c = SsaOffscreen::default();
    for r in find_ssa_refs(text, name) {
        if r.line < view_start {
            match r.kind {
                SsaRefKind::Def => c.above_defs += 1,
                SsaRefKind::Use => c.above_uses += 1,
            }
        } else if r.line >= view_end {
            match r.kind {
                SsaRefKind::Def => c.below_defs += 1,
                SsaRefKind::Use => c.below_uses += 1,
            }
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_ssa_refs_distinguishes_def_and_use() {
        let text = "    v0 = const u64 1\n    v1 = add v0, v0\n";
        let refs = find_ssa_refs(text, "v0");
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].kind, SsaRefKind::Def);
        assert_eq!(refs[0].line, 0);
        assert_eq!(refs[1].kind, SsaRefKind::Use);
        assert_eq!(refs[2].kind, SsaRefKind::Use);
        let off = count_ssa_offscreen(text, "v0", 1, 2);
        assert_eq!(off.above_defs, 1);
        assert_eq!(off.above_uses, 0);
        assert_eq!(off.below_defs, 0);
        assert_eq!(off.below_uses, 0);
        let off2 = count_ssa_offscreen(text, "v0", 0, 1);
        assert_eq!(off2.below_uses, 2);
    }

    #[test]
    fn symbol_at_col_finds_local_and_method() {
        assert!(!SSA_PALETTE.is_empty());
        assert_eq!(SSA_PALETTE.len(), SYMBOL_PALETTE.len());
        let text = "    fn main() -> u64 {\n        local u64 x\n        entry():\n        v0 = get_local __ptr u64, x\n        v1 = call foo(v0)\n    }\n    fn foo(a: u64) -> u64 {\n        ret u64 a\n    }\n";
        assert_eq!(local_decl_name("        local u64 x").as_deref(), Some("x"));
        assert_eq!(get_local_name("        v0 = get_local __ptr u64, x").as_deref(), Some("x"));
        assert_eq!(fn_def_name("    fn main() -> u64 {").as_deref(), Some("main"));
        assert_eq!(call_name("        v1 = call foo(v0)").as_deref(), Some("foo"));
        assert_eq!(symbol_at_col("        local u64 x", 18, text).as_deref(), Some("x"));
        assert_eq!(symbol_at_col("    fn main() -> u64 {", 7, text).as_deref(), Some("main"));
        assert_eq!(symbol_at_col("        v1 = call foo(v0)", 18, text).as_deref(), Some("foo"));
        let (above, below) = count_token_offscreen(text, "x", 2, 3);
        assert!(above >= 1);
        assert!(below >= 1);
    }

    #[test]
    fn ssa_at_col_finds_register() {
        assert_eq!(ssa_at_col("    v0 = const u64 1", 4).as_deref(), Some("v0"));
        assert_eq!(ssa_at_col("    v12v1 = add v0, v1", 6).as_deref(), Some("v12v1"));
        // "    add v0" → 'v' is at column 8
        assert_eq!(ssa_at_col("    add v0, v1", 8).as_deref(), Some("v0"));
        assert!(ssa_at_col("    local u64 x", 4).is_none());
    }

    #[test]
    fn ssa_highlight_overrides_syntax_background() {
        let mut lines = highlight_ir_with_search("v0 = add v0, v1\n", false, "", None, true);
        apply_ssa_highlights(&mut lines, &[("v0".into(), Color::Rgb(1, 2, 3))]);
        let has_bg = lines[0].spans.iter().any(|s| {
            s.content.contains('v') && s.style.bg == Some(Color::Rgb(1, 2, 3))
        });
        assert!(has_bg, "expected v0 spans to carry SSA background");
    }

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
    use crate::parse::{strip_metadata_and_version, strip_version_suffix, MdMode, VersionMode};

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
        let _ = highlight_ir_with_search(&stripped, false, "v0", Some(0), true);
    }

    #[test]
    fn highlighting_real_ir_after_version_strip() {
        use std::path::PathBuf;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        if !root.exists() {
            return;
        }
        for entry in std::fs::read_dir(&root).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("ir") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else {
                continue;
            };
            let stripped = strip_version_suffix(&text);
            let _ = highlight_ir(&stripped, false);
            let _ = highlight_ir(&stripped, true);
        }
    }
}
