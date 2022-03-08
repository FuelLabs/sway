//! Helpers for validating and checking names like package and organization names.
// This is based on https://github.com/rust-lang/cargo/blob/489b66f2e458404a10d7824194d3ded94bc1f4e4/src/cargo/util/restricted_names.rs

use anyhow::{bail, Result};
use std::path::Path;

/// Returns `true` if the name contains non-ASCII characters.
pub fn is_non_ascii_name(name: &str) -> bool {
    name.chars().any(|ch| ch > '\x7f')
}

/// Rust keywords, further bikeshedding necessary to determine a complete set of Sway keywords
pub fn is_keyword(name: &str) -> bool {
    // See https://doc.rust-lang.org/reference/keywords.html
    [
        "Self", "abstract", "as", "await", "become", "box", "break", "const", "continue", "dep",
        "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
        "let", "loop", "macro", "match", "move", "mut", "override", "priv", "pub", "ref", "return",
        "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof", "unsafe",
        "unsized", "use", "virtual", "where", "while", "yield",
    ]
    .contains(&name)
}

/// These names cannot be used on Windows, even with an extension.
pub fn is_windows_reserved(name: &str) -> bool {
    [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ]
    .contains(&name.to_ascii_lowercase().as_str())
}

/// These names conflict with library, macro or heap allocation suffixes, or keywords.
pub fn is_conflicting_suffix(name: &str) -> bool {
    ["alloc", "proc_macro", "proc-macro"].contains(&name)
}

// Bikeshedding necessary to determine if relevant
/// An artifact with this name will conflict with one of forc's build directories.
pub fn is_conflicting_artifact_name(name: &str) -> bool {
    ["deps", "examples", "build", "incremental"].contains(&name)
}

/// Check the package name for invalid characters.
pub fn contains_invalid_char(name: &str, use_case: &str) -> Result<()> {
    let mut chars = name.chars();
    if let Some(ch) = chars.next() {
        if ch.is_digit(10) {
            // A specific error for a potentially common case.
            bail!(
                "the name `{name}` cannot be used as a {use_case}, \
                the name cannot start with a digit"
            );
        }
        if !(unicode_xid::UnicodeXID::is_xid_start(ch) || ch == '_') {
            bail!(
                "invalid character `{ch}` in {use_case}: `{name}`, \
                the first character must be a Unicode XID start character \
                (most letters or `_`)"
            );
        }
    }
    for ch in chars {
        if !(unicode_xid::UnicodeXID::is_xid_continue(ch) || ch == '-') {
            bail!(
                "invalid character `{ch}` in {use_case}: `{name}`, \
                characters must be Unicode XID characters \
                (numbers, `-`, `_`, or most letters)"
            );
        }
    }
    if name.is_empty() {
        bail!(
            "{use_case} cannot be left empty, \
            please use a valid name"
        );
    }
    Ok(())
}

/// Check the entire path for names reserved in Windows.
pub fn is_windows_reserved_path(path: &Path) -> bool {
    path.iter()
        .filter_map(|component| component.to_str())
        .any(|component| {
            let stem = component.split('.').next().unwrap();
            is_windows_reserved(stem)
        })
}

/// Returns `true` if the name contains any glob pattern wildcards.
pub fn is_glob_pattern<T: AsRef<str>>(name: T) -> bool {
    name.as_ref().contains(&['*', '?', '[', ']'][..])
}

#[test]
fn test_invalid_char() {
    assert_eq!(
        contains_invalid_char("test#proj", "package name").map_err(|e| e.to_string()),
        std::result::Result::Err(
            "invalid character `#` in package name: `test#proj`, \
        characters must be Unicode XID characters \
        (numbers, `-`, `_`, or most letters)"
                .into()
        )
    );

    assert_eq!(
        contains_invalid_char("test proj", "package name").map_err(|e| e.to_string()),
        std::result::Result::Err(
            "invalid character ` ` in package name: `test proj`, \
        characters must be Unicode XID characters \
        (numbers, `-`, `_`, or most letters)"
                .into()
        )
    );

    assert_eq!(
        contains_invalid_char("", "package name").map_err(|e| e.to_string()),
        std::result::Result::Err(
            "package name cannot be left empty, \
        please use a valid name"
                .into()
        )
    );

    assert!(matches!(
        contains_invalid_char("test_proj", "package name"),
        std::result::Result::Ok(())
    ));
}
