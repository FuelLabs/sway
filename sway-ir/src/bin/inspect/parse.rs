//! Parsing and IR text utilities shared by classic and TUI modes.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PrintMode {
    #[default]
    Ir,
    Diff,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub(crate) enum MdMode {
    #[default]
    AsParsed,
    With,
    Without,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub(crate) enum VersionMode {
    #[default]
    AsParsed,
    With,
    Without,
}

pub(crate) fn prepare_ir_text(
    filter_fn: &[String],
    metadata: MdMode,
    version: VersionMode,
    ir: &ParsedIr,
) -> Option<String> {
    if filter_fn.is_empty() {
        return Some(strip_metadata_and_version(&ir.body, metadata, version));
    }

    let lines = ir.body.lines().collect::<Vec<_>>();
    let mut text = String::new();
    for decl in find_functions(&ir.body) {
        let matches = filter_fn.iter().any(|f| decl.name.contains(f));
        if !matches {
            continue;
        }
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(&lines[decl.start..=decl.end].join("\n"));
    }

    if text.is_empty() {
        None
    } else {
        Some(strip_metadata_and_version(&text, metadata, version))
    }
}


pub(crate) fn strip_metadata_and_version(body: &str, metadata: MdMode, version: VersionMode) -> String {
    let mut s = body.to_string();
    if version == VersionMode::Without {
        s = strip_version_suffix(&s);
    }
    if metadata == MdMode::Without {
        s = strip_metadata(&s);
    }
    s.trim_end().to_string()
}

pub(crate) struct ParsedIr {
    /// Package / compilation unit label from the nearest preceding
    /// `Compiling …` / `Building …` line (or `"(unknown)"`).
    pub project: String,
    pub pass_name: String,
    pub body: String,
}

impl ParsedIr {
    /// The `// IR: Initial` header marks the start of a new compilation unit.
    pub(crate) fn is_initial(&self) -> bool {
        self.pass_name == "Initial"
    }
}

fn push_parsed(out: &mut Vec<ParsedIr>, project: String, pass_name: String, body: Vec<&str>) {
    out.push(ParsedIr {
        project,
        pass_name,
        body: body.join("\n"),
    });
}

pub(crate) fn parse(cleaned: &str) -> Vec<ParsedIr> {
    let mut parsed_irs = Vec::new();
    // (project, pass_name, body lines)
    let mut current: Option<(String, String, Vec<&str>)> = None;
    let mut current_project = String::from("(unknown)");

    for line in cleaned.lines() {
        let trimmed = line.trim_start();

        // Track package boundaries even between dumps (when `current` is None).
        if let Some(rest) = trimmed
            .strip_prefix("Compiling ")
            .or_else(|| trimmed.strip_prefix("Building "))
        {
            current_project = rest.trim().to_string();
            if let Some((project, name, body)) = current.take() {
                push_parsed(&mut parsed_irs, project, name, body);
            }
            continue;
        }

        // The `// IR:` header may be indented (some dumps pad every line with
        // leading whitespace), so match against the trimmed-start line rather
        // than requiring the line to begin exactly with `// IR: `.  The body
        // lines below are still pushed verbatim, preserving their indentation.
        if let Some(rest) = trimmed.strip_prefix("// IR: ") {
            if let Some((project, name, body)) = current.take() {
                push_parsed(&mut parsed_irs, project, name, body);
            }
            current = Some((
                current_project.clone(),
                rest.trim().to_string(),
                Vec::new(),
            ));
            continue;
        }

        if let Some((_, _, body)) = current.as_mut() {
            body.push(line);
        }
    }

    if let Some((project, name, body)) = current {
        push_parsed(&mut parsed_irs, project, name, body);
    }

    parsed_irs
}

pub(crate) struct FuncDecl {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

pub(crate) fn find_functions(body: &str) -> Vec<FuncDecl> {
    let lines: Vec<&str> = body.lines().collect();
    let mut decls = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let Some(name) = function_decl_name(line) else {
            continue;
        };
        let indent = leading_spaces(line);
        // The matching closing brace is the next line at the same indentation
        // whose content is just `}`.  Function bodies are indented one level
        // deeper, so braces inside the body won't match.
        let mut end = idx;
        for (j, l) in lines.iter().enumerate().skip(idx + 1) {
            if leading_spaces(l) == indent && l.trim() == "}" {
                end = j;
                break;
            }
        }
        decls.push(FuncDecl {
            name,
            start: idx,
            end,
        });
    }

    decls
}

/// If `line` is a function declaration, return the declared function name.
pub(crate) fn function_decl_name(line: &str) -> Option<String> {
    let mut rest = line.trim_start();

    loop {
        let stripped = rest
            .strip_prefix("pub ")
            .or_else(|| rest.strip_prefix("entry_orig "))
            .or_else(|| rest.strip_prefix("entry "))
            .or_else(|| rest.strip_prefix("fallback "));
        match stripped {
            Some(s) => rest = s,
            None => break,
        }
    }

    let rest = rest.strip_prefix("fn ")?;
    let name_end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    if name_end == 0 {
        return None;
    }
    Some(rest[..name_end].to_string())
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|b| *b == b' ').count()
}

#[inline]
fn is_id_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[derive(Default, Clone)]
pub(crate) struct FuncStats {
    pub args: usize,
    pub blocks: usize,
    pub instructions: usize,
    pub ops: BTreeMap<String, usize>,
}

impl FuncStats {
    pub(crate) fn compute_stats(text: &str) -> FuncStats {
        let lines: Vec<&str> = text.lines().collect();
        let mut stats = FuncStats::default();
        for decl in find_functions(text) {
            stats.args += Self::count_args(lines[decl.start]);
            let mut asm_depth = 0usize;
            for &line in &lines[decl.start + 1..=decl.end] {
                let t = line.trim();
                if t.is_empty() || t.starts_with("//") {
                    continue;
                }
                if asm_depth > 0 {
                    if t == "}" {
                        asm_depth -= 1;
                    }
                    continue;
                }
                if t == "}" || t.starts_with("local ") {
                    continue;
                }
                if t.ends_with(':') {
                    stats.blocks += 1;
                    continue;
                }
                stats.instructions += 1;
                *stats.ops.entry(Self::instruction_op(t)).or_insert(0) += 1;
                if t.ends_with('{') {
                    asm_depth += 1;
                }
            }
        }
        stats
    }

    fn count_args(decl: &str) -> usize {
        let bytes = decl.as_bytes();
        let mut i = match bytes.iter().position(|b| *b == b'(') {
            Some(p) => p + 1,
            None => return 0,
        };
        let mut depth = 1;
        let mut commas = 0;
        let mut content = false;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'(' => {
                    depth += 1;
                    content = true;
                }
                b')' => {
                    depth -= 1;
                    if depth > 0 {
                        content = true;
                    }
                }
                b',' if depth == 1 => {
                    commas += 1;
                    content = true;
                }
                b if !b.is_ascii_whitespace() => content = true,
                _ => {}
            }
            i += 1;
        }
        if content {
            commas + 1
        } else {
            0
        }
    }

    fn instruction_op(line: &str) -> String {
        let rest = match line.find(" = ") {
            Some(idx) => &line[idx + 3..],
            None => line,
        };
        let tok = rest.split_whitespace().next().unwrap_or("");
        tok.split('(').next().unwrap_or("").to_string()
    }

    fn stat_with_delta(cur: usize, prev: Option<usize>) -> String {
        let base = cur.to_string();
        match prev {
            Some(p) => {
                let d = cur as isize - p as isize;
                if d != 0 {
                    format!("{base} ({:+})", d)
                } else {
                    base
                }
            }
            None => base,
        }
    }

    pub(crate) fn print_fn_stats<W: Write>(
        self,
        out: &mut W,
        prev: Option<&FuncStats>,
    ) -> std::io::Result<()> {
        let mut line = String::from("// Fn Stats:");
        line.push_str(&format!(
            " args={}",
            Self::stat_with_delta(self.args, prev.map(|p| p.args))
        ));
        line.push_str(&format!(
            " blocks={}",
            Self::stat_with_delta(self.blocks, prev.map(|p| p.blocks))
        ));
        line.push_str(&format!(
            " instructions={}",
            Self::stat_with_delta(self.instructions, prev.map(|p| p.instructions))
        ));

        // Union of opcodes (current + previous), sorted for stable output.  When
        // diffing, opcodes that disappeared show a count of 0 with a negative delta.
        let mut names: BTreeSet<&str> = self.ops.keys().map(String::as_str).collect();
        if let Some(p) = prev {
            for k in p.ops.keys() {
                names.insert(k.as_str());
            }
        }
        for name in names {
            let c = self.ops.get(name).copied().unwrap_or(0);
            let p = prev.and_then(|ps| ps.ops.get(name).copied());
            line.push_str(&format!(" {}={}", name, Self::stat_with_delta(c, p)));
        }

        writeln!(out, "{line}")
    }
}

pub(crate) fn strip_version_suffix(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut copy_from = 0usize;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'v' && (i == 0 || !is_id_byte(bytes[i - 1])) {
            // Try to match `v<digits>v<digits>` as a whole token.
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let idx_end = j;
            if j > i + 1 && j < bytes.len() && bytes[j] == b'v' {
                let mut k = j + 1;
                while k < bytes.len() && bytes[k].is_ascii_digit() {
                    k += 1;
                }
                // The version part must end at a non-identifier byte, otherwise
                // this isn't a `v<idx>v<version>` value name.
                if k > j + 1 && (k == bytes.len() || !is_id_byte(bytes[k])) {
                    // Flush everything up to the version suffix, keep `v{idx}`.
                    out.push_str(std::str::from_utf8(&bytes[copy_from..idx_end]).unwrap());
                    copy_from = k;
                    i = k;
                    continue;
                }
            }
        }
        i += 1;
    }

    out.push_str(std::str::from_utf8(&bytes[copy_from..]).unwrap());
    out
}

pub(crate) fn strip_metadata(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for line in input.lines() {
        // Drop metadata definition lines, e.g. `!1 = SomeTag ...`.
        if is_metadata_def_line(line) {
            continue;
        }
        out.push_str(&strip_inline_metadata(line));
        out.push('\n');
    }

    // `input.lines()` drops a trailing newline; preserve the original shape by
    // trimming the extra newline we added for the last line.
    out.trim_end_matches('\n').to_string()
}

pub(crate) fn is_metadata_def_line(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    if bytes.first() != Some(&b'!') {
        return false;
    }
    i += 1;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    // No digits after `!` => not a metadata def.
    if i == 1 {
        return false;
    }
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    i < bytes.len() && bytes[i] == b'='
}

fn strip_inline_metadata(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut copy_from = 0usize;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            // Flush the text up to (but not including) the `!N` reference,
            // trimming a trailing separator (comma + surrounding spaces) so we
            // don't leave dangling `,` or extra spaces behind.
            let mut flush_end = i;
            while flush_end > copy_from && bytes[flush_end - 1] == b' ' {
                flush_end -= 1;
            }
            if flush_end > copy_from && bytes[flush_end - 1] == b',' {
                flush_end -= 1;
            }
            out.push_str(std::str::from_utf8(&bytes[copy_from..flush_end]).unwrap());

            // Consume the `!N` token.
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            copy_from = j;
            i = j;
        } else {
            i += 1;
        }
    }

    out.push_str(std::str::from_utf8(&bytes[copy_from..]).unwrap());
    out
}

pub(crate) fn print_diff<W: Write>(
    out: &mut W,
    ops: &[prettydiff::basic::DiffOp<&str>],
    color: bool,
) -> std::io::Result<()> {
    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const RESET: &str = "\x1b[0m";

    let write_lines = |out: &mut W, prefix: &str, lines: &[&str], c: &str| -> std::io::Result<()> {
        for line in lines {
            if color && !c.is_empty() {
                writeln!(out, "{c}{prefix}{line}{RESET}")?;
            } else {
                writeln!(out, "{prefix}{line}")?;
            }
        }
        Ok(())
    };

    for op in ops {
        match op {
            prettydiff::basic::DiffOp::Equal(lines) => write_lines(out, "  ", lines, "")?,
            prettydiff::basic::DiffOp::Insert(lines) => write_lines(out, "+ ", lines, GREEN)?,
            prettydiff::basic::DiffOp::Remove(lines) => write_lines(out, "- ", lines, RED)?,
            prettydiff::basic::DiffOp::Replace(removed, inserted) => {
                write_lines(out, "- ", removed, RED)?;
                write_lines(out, "+ ", inserted, GREEN)?;
            }
        }
    }

    Ok(())
}

pub(crate) fn diff_stats(ops: &[prettydiff::basic::DiffOp<&str>]) -> (usize, usize) {
    let mut adds = 0;
    let mut removes = 0;
    for op in ops {
        match op {
            prettydiff::basic::DiffOp::Insert(lines) => adds += lines.len(),
            prettydiff::basic::DiffOp::Remove(lines) => removes += lines.len(),
            prettydiff::basic::DiffOp::Replace(removed, inserted) => {
                removes += removed.len();
                adds += inserted.len();
            }
            prettydiff::basic::DiffOp::Equal(_) => {}
        }
    }
    (adds, removes)
}

pub(crate) fn print_diff_stats<W: Write>(out: &mut W, adds: usize, removes: usize) -> std::io::Result<()> {
    writeln!(out, "// Diff Stats: adds={adds} removes={removes}")
}

/// Spawn `cmd` under a shell with stdout piped (stderr merged via `2>&1`).
///
/// The caller owns the [`std::process::Child`] and can kill it (e.g. on Ctrl-C).
pub(crate) fn spawn_shell(cmd: &str) -> anyhow::Result<std::process::Child> {
    use std::process::{Command, Stdio};

    Command::new("bash")
        .arg("-lc")
        .arg(format!("{{ {cmd}\n}} 2>&1"))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn shell for `--cmd`: {e}"))
}

/// Spawn `cmd` under a shell and capture merged stdout + stderr.
///
/// Used for `--cmd` in classic mode. Stderr is redirected into stdout by the
/// shell (`2>&1`) so both streams are captured in order as a single pipe.
pub(crate) fn capture_shell(cmd: &str) -> anyhow::Result<String> {
    use std::io::Read;

    let mut child = spawn_shell(cmd)?;
    let mut text = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout
            .read_to_string(&mut text)
            .map_err(|e| anyhow::anyhow!("failed to read `--cmd` output: {e}"))?;
    }
    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("failed to wait for `--cmd`: {e}"))?;

    if text.trim().is_empty() && !status.success() {
        let code = status.code().unwrap_or(-1);
        anyhow::bail!("`--cmd` exited with status {code} and produced no output");
    }

    Ok(text)
}

pub(crate) fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut copy_from = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == 0x1b {
            out.extend_from_slice(&bytes[copy_from..i]);
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                // CSI: skip until the final byte in 0x40..=0x7e.
                i += 1;
                while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            } else if i < bytes.len() && bytes[i] == b']' {
                // OSC: skip until BEL or ST (ESC backslash).
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 0x07 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            } else if i < bytes.len() {
                // Other escape sequence: skip the next byte.
                i += 1;
            }
            copy_from = i;
        } else {
            i += 1;
        }
    }
    out.extend_from_slice(&bytes[copy_from..]);

    // SAFETY: see the doc comment; only ASCII escape sequences were removed and
    // the remaining bytes are copied verbatim, so `out` is valid UTF-8.
    String::from_utf8(out).unwrap()
}

/// Prints the final IR text, optionally prefixing each line that carries
/// metadata with the original source code it refers to (see `--print source`).
pub(crate) fn print_final_ir<W: Write>(
    out: &mut W,
    text: &str,
    source_map: Option<&mut SourceMap>,
) -> std::io::Result<()> {
    let Some(sm) = source_map else {
        return writeln!(out, "{}", text);
    };

    for line in text.lines() {
        // Skip metadata *definition* lines (`!N = ...`); only IR lines that
        // *reference* metadata get a source prefix.
        if !is_metadata_def_line(line) {
            if let Some((path, start, end, src)) = sm.source_for_line(line) {
                if !src.is_empty() {
                    writeln!(out, "// src: {path} [{start}..{end})")?;
                    for src_line in src.lines() {
                        writeln!(out, "  | {src_line}")?;
                    }
                }
            }
        }
        writeln!(out, "{}", line)?;
    }

    Ok(())
}

/// Parsed metadata definitions from a single IR dump, used to resolve inline
/// `!N` references on IR lines back to the source spans they point at.
pub(crate) struct SourceMap {
    entries: BTreeMap<u64, MdValue>,
    /// Metadata file index -> resolved source contents (or `None` if the file
    /// could not be read).  Keyed by index so each file/inline blob is read at
    /// most once, regardless of how many spans reference it.
    content_cache: BTreeMap<u64, Option<String>>,
}

#[derive(Debug, Clone)]
pub(crate) enum MdValue {
    /// `!N = "/path/to/file.sw"` — a source file location.
    SourceId(String),
    /// `!N = inline "<code>"` — the source text inlined directly (emitted by
    /// the printer for `<autogenerated>` files that don't exist on disk).
    Inline(String),
    /// `!N = <tag> !M <start> <end>` — a source span (tags: `span`,
    /// `fn_name_span`, `fn_call_path_span`, ...).  `file` is the index of the
    /// referenced `SourceId`/`Inline`.
    Span { file: u64, start: u64, end: u64 },
    /// `!N = (!a !b ...)` — a list of metadata indices.
    List(Vec<u64>),
    /// `!N = !M` — a bare index reference (`Metadatum::Index`).
    Ref(u64),
    /// Anything else (purity, decl_index, ...); not source-carrying.
    Other,
}

pub(crate) fn parse_source_map(body: &str) -> SourceMap {
    let mut entries = BTreeMap::new();
    for line in body.lines() {
        if let Some((idx, value)) = parse_metadata_def(line) {
            entries.insert(idx, value);
        }
    }
    SourceMap {
        entries,
        content_cache: BTreeMap::new(),
    }
}

/// Whether the input contains any `!N = ...` metadata definition lines.
/// Without them, `--print source` cannot resolve inline `!N` references.
pub(crate) fn has_metadata_defs(cleaned: &str) -> bool {
    cleaned.lines().any(is_metadata_def_line)
}

/// Parse a `!N = <value>` metadata definition line into its index and value.
fn parse_metadata_def(line: &str) -> Option<(u64, MdValue)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'!' {
        return None;
    }
    i += 1;
    let num_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let num: u64 = std::str::from_utf8(&bytes[num_start..i])
        .ok()?
        .parse()
        .ok()?;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'=' {
        return None;
    }
    i += 1;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    Some((num, parse_md_value(&line[i..])))
}

fn parse_md_value(s: &str) -> MdValue {
    let s = s.trim();

    // Inlined source: `inline "<code>"` — emitted by the printer for
    // `<autogenerated>` files so the source is available without the file on
    // disk.  Must be matched before the generic struct form below, which would
    // otherwise parse `inline` as a struct tag.
    if let Some(rest) = s.strip_prefix("inline") {
        let rest = rest.trim_start();
        if let Some(inner) = rest.strip_prefix('"') {
            if let Some(end) = inner.rfind('"') {
                return MdValue::Inline(unescape_debug_string(&inner[..end]));
            }
        }
        return MdValue::Other;
    }

    // Quoted string: a `SourceId` file location, e.g. `"/path/to/file.sw"`.
    if let Some(inner) = s.strip_prefix('"') {
        if let Some(end) = inner.rfind('"') {
            return MdValue::SourceId(unescape_debug_string(&inner[..end]));
        }
        return MdValue::Other;
    }

    // List: `(!a !b ...)`.
    if let Some(inner) = s.strip_prefix('(') {
        if let Some(inner) = inner.strip_suffix(')') {
            let idxs = inner
                .split_whitespace()
                .filter_map(|tok| tok.strip_prefix('!').and_then(|n| n.parse::<u64>().ok()))
                .collect();
            return MdValue::List(idxs);
        }
        return MdValue::Other;
    }

    // Bare index reference: `!M`.
    if let Some(rest) = s.strip_prefix('!') {
        if let Ok(n) = rest.parse::<u64>() {
            return MdValue::Ref(n);
        }
    }

    // Struct: `<tag> <field> ...`.  A span-shaped struct has exactly three
    // fields: `!M <int> <int>`.
    let mut tokens = s.split_whitespace();
    let (Some(tag), Some(file_tok), Some(start_tok), Some(end_tok)) =
        (tokens.next(), tokens.next(), tokens.next(), tokens.next())
    else {
        return MdValue::Other;
    };
    if tokens.next().is_some() {
        return MdValue::Other;
    }
    // The tag must be an identifier, not a `!N`/integer.
    if tag.starts_with('!') || tag.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return MdValue::Other;
    }
    let file = match file_tok
        .strip_prefix('!')
        .and_then(|n| n.parse::<u64>().ok())
    {
        Some(n) => n,
        None => return MdValue::Other,
    };
    let start: u64 = match start_tok.parse() {
        Ok(n) => n,
        Err(_) => return MdValue::Other,
    };
    let end: u64 = match end_tok.parse() {
        Ok(n) => n,
        Err(_) => return MdValue::Other,
    };
    MdValue::Span { file, start, end }
}

/// Unescape the content of a Rust debug-quoted string (the bytes between the
/// outer `"`s).  `SourceId`s are printed with `{:?}` on a `PathBuf`, which
/// quotes the path and escapes `\` and `"` (and the usual control chars).
fn unescape_debug_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('0') => out.push('\0'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl SourceMap {
    /// Resolve the source snippet for an IR line.  Every inline `!N` reference
    /// is expanded to *all* spans it (transitively) reaches — lists such as
    /// `!1883 = (!1879 !1880 !1882)` commonly mix spans pointing at different
    /// files — and the first span whose file/inline source can be located is
    /// used.  Returns `None` if the line has no metadata references, no span,
    /// or none of the spans' sources can be obtained.
    fn source_for_line(&mut self, line: &str) -> Option<(String, u64, u64, String)> {
        for idx in find_md_refs(line) {
            for (file_idx, start, end) in self.collect_spans(idx, &mut BTreeSet::new()) {
                // Compute the label first (immutable borrow) so the mutable
                // borrow from `content_for_idx` below isn't held across it.
                let label = self.file_label(file_idx);
                if let Some(content) = self.content_for_idx(file_idx) {
                    if let Some(text) = expand_to_lines(content, start as usize, end as usize) {
                        if !text.trim().is_empty() {
                            return Some((label, start, end, text.to_string()));
                        }
                    }
                }
            }
        }
        None
    }

    /// Resolve the source content for a metadata file index, caching the
    /// result.  For `SourceId` the file is read from disk; for `Inline` the
    /// embedded text is used directly.  Returns `None` if unreadable.
    fn content_for_idx(&mut self, file_idx: u64) -> Option<&String> {
        if !self.content_cache.contains_key(&file_idx) {
            let content = match self.entries.get(&file_idx) {
                Some(MdValue::SourceId(path)) => std::fs::read_to_string(path).ok(),
                Some(MdValue::Inline(code)) => Some(code.clone()),
                _ => None,
            };
            self.content_cache.insert(file_idx, content);
        }
        self.content_cache.get(&file_idx).and_then(|c| c.as_ref())
    }

    /// A human-readable label for the source of a span, used in the `// src:`
    /// header.  For `SourceId` this is the path; for `Inline` there is no path,
    /// so a placeholder is shown.
    fn file_label(&self, file_idx: u64) -> String {
        match self.entries.get(&file_idx) {
            Some(MdValue::SourceId(path)) => path.clone(),
            Some(MdValue::Inline(_)) => "<inline>".to_string(),
            _ => "<unknown>".to_string(),
        }
    }

    /// Depth-first collect every `(file_idx, start, end)` span reachable from
    /// `idx`, recursing through lists and `Ref` indirection.  `visited` guards
    /// against cycles in the metadata graph.
    fn collect_spans(&self, idx: u64, visited: &mut BTreeSet<u64>) -> Vec<(u64, u64, u64)> {
        let mut out = Vec::new();
        self.collect_spans_into(idx, visited, &mut out);
        out
    }

    fn collect_spans_into(
        &self,
        idx: u64,
        visited: &mut BTreeSet<u64>,
        out: &mut Vec<(u64, u64, u64)>,
    ) {
        if !visited.insert(idx) {
            return;
        }
        match self.entries.get(&idx) {
            Some(MdValue::Span { file, start, end }) => {
                if let Some(file_idx) = self.resolve_file_idx(*file, &mut BTreeSet::new()) {
                    out.push((file_idx, *start, *end));
                }
            }
            Some(MdValue::List(idxs)) => {
                for &i in idxs {
                    self.collect_spans_into(i, visited, out);
                }
            }
            Some(MdValue::Ref(r)) => self.collect_spans_into(*r, visited, out),
            _ => {}
        }
    }

    /// Follow `Ref` indirection to the concrete `SourceId`/`Inline` index that
    /// a span's `file` field refers to.  Uses a fresh cycle guard so a file
    /// index shared by several spans in the same list resolves for each.
    fn resolve_file_idx(&self, idx: u64, visited: &mut BTreeSet<u64>) -> Option<u64> {
        if !visited.insert(idx) {
            return None;
        }
        match self.entries.get(&idx)? {
            MdValue::SourceId(_) | MdValue::Inline(_) => Some(idx),
            MdValue::Ref(r) => self.resolve_file_idx(*r, visited),
            _ => None,
        }
    }
}

/// Expand a `[start, end)` byte range to whole-line boundaries of `content`,
/// returning the full source line(s) that intersect the span (trailing newline
/// excluded).  This gives readable context around a span that may otherwise be
/// just a few tokens (e.g. an inline-asm mnemonic).
pub(crate) fn expand_to_lines(content: &str, start: usize, end: usize) -> Option<&str> {
    if end < start || start > content.len() || end > content.len() {
        return None;
    }
    // Start of the line containing `start`: right after the previous newline.
    let line_start = content
        .get(..start)?
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    // End of the line containing the last span byte (`end - 1`): the next
    // newline at or after `end`, or end-of-file.
    let suffix = content.get(end.saturating_sub(1)..)?;
    let rel = suffix.find('\n').unwrap_or(suffix.len());
    let line_end = end.saturating_sub(1) + rel;
    content.get(line_start..line_end)
}

/// Find all inline `!N` metadata references in a line.
pub(crate) fn find_md_refs(line: &str) -> Vec<u64> {
    let bytes = line.as_bytes();
    let mut refs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if let Ok(n) = std::str::from_utf8(&bytes[i + 1..j])
                .unwrap_or("")
                .parse::<u64>()
            {
                refs.push(n);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    refs
}



#[cfg(test)]
mod strip_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn strip_version_suffix_basic() {
        assert_eq!(strip_version_suffix("v0v1"), "v0");
        assert_eq!(strip_version_suffix("foo_v0v1"), "foo_v0v1");
        assert_eq!(strip_version_suffix("a v10v20 b"), "a v10 b");
    }

    #[test]
    fn strip_version_suffix_unicode() {
        assert_eq!(strip_version_suffix("→ v0v1"), "→ v0");
        let _ = strip_version_suffix("日本語 v12v3");
        let _ = strip_version_suffix("●v1v2");
    }

    #[test]
    fn strip_version_on_repo_ir_files_does_not_panic() {
        let roots = [
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test/src/ir_generation/tests"),
        ];
        let mut n = 0usize;
        for root in roots {
            if !root.exists() {
                continue;
            }
            for entry in walkdir(&root) {
                let Ok(text) = std::fs::read_to_string(&entry) else { continue };
                let _ = strip_version_suffix(&text);
                let _ = strip_metadata(&text);
                let _ = strip_metadata_and_version(&text, MdMode::Without, VersionMode::Without);
                n += 1;
            }
        }
        assert!(n > 0, "expected to find at least one .ir file");
    }

    fn walkdir(root: &std::path::Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|s| s.to_str()) == Some("ir") {
                    out.push(p);
                }
            }
        }
        out
    }
}
