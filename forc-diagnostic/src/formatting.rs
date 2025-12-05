//! Diagnostic formatting utilities using annotate-snippets.

use annotate_snippets::{
    renderer::{AnsiColor, Style},
    Annotation, AnnotationType, Renderer, Slice, Snippet, SourceAnnotation,
};
use std::collections::HashSet;
use sway_error::diagnostic::{Diagnostic, Issue, Label, LabelType, Level};
use sway_types::{LineCol, LineColRange, Span};

/// Creates [Renderer] for printing warnings and errors.
///
/// To ensure the same styling of printed warnings and errors across all the tools,
/// always use this function to create [Renderer]s.
pub fn create_diagnostics_renderer() -> Renderer {
    // For the diagnostic messages we use bold and bright colors.
    // Note that for the summaries of warnings and errors we use
    // their regular equivalents which are defined in this package.
    Renderer::styled()
        .warning(
            Style::new()
                .bold()
                .fg_color(Some(AnsiColor::BrightYellow.into())),
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(AnsiColor::BrightRed.into())),
        )
}

pub fn format_diagnostic(diagnostic: &Diagnostic) {
    /// Temporary switch for testing the feature.
    /// Keep it false until we decide to fully support the diagnostic codes.
    const SHOW_DIAGNOSTIC_CODE: bool = false;

    if diagnostic.is_old_style() {
        format_old_style_diagnostic(diagnostic.issue());
        return;
    }

    let mut label = String::new();
    get_title_label(diagnostic, &mut label);

    let snippet_title = Some(Annotation {
        label: Some(label.as_str()),
        id: if SHOW_DIAGNOSTIC_CODE {
            diagnostic.reason().map(|reason| reason.code())
        } else {
            None
        },
        annotation_type: diagnostic_level_to_annotation_type(diagnostic.level()),
    });

    let mut snippet_slices = Vec::<Slice<'_>>::new();

    // We first display labels from the issue file...
    if diagnostic.issue().is_in_source() {
        snippet_slices.push(construct_slice(diagnostic.labels_in_issue_source()))
    }

    // ...and then all the remaining labels from the other files.
    for source_path in diagnostic.related_sources(false) {
        snippet_slices.push(construct_slice(diagnostic.labels_in_source(source_path)))
    }

    let mut snippet_footer = Vec::<Annotation<'_>>::new();
    for help in diagnostic.help() {
        snippet_footer.push(Annotation {
            id: None,
            label: Some(help),
            annotation_type: AnnotationType::Help,
        });
    }

    let snippet = Snippet {
        title: snippet_title,
        slices: snippet_slices,
        footer: snippet_footer,
    };

    let renderer = create_diagnostics_renderer();
    match diagnostic.level() {
        Level::Info => tracing::info!("{}\n____\n", renderer.render(snippet)),
        Level::Warning => tracing::warn!("{}\n____\n", renderer.render(snippet)),
        Level::Error => tracing::error!("{}\n____\n", renderer.render(snippet)),
    }

    fn format_old_style_diagnostic(issue: &Issue) {
        let annotation_type = label_type_to_annotation_type(issue.label_type());

        let snippet_title = Some(Annotation {
            label: if issue.is_in_source() {
                None
            } else {
                Some(issue.text())
            },
            id: None,
            annotation_type,
        });

        let mut snippet_slices = vec![];
        if issue.is_in_source() {
            let span = issue.span();
            let input = span.input();
            let mut start_pos = span.start();
            let mut end_pos = span.end();
            let LineColRange { mut start, end } = span.line_col_one_index();
            let input = construct_window(&mut start, end, &mut start_pos, &mut end_pos, input);

            let slice = Slice {
                source: input,
                line_start: start.line,
                // Safe unwrap because the issue is in source, so the source path surely exists.
                origin: Some(issue.source_path().unwrap().as_str()),
                fold: false,
                annotations: vec![SourceAnnotation {
                    label: issue.text(),
                    annotation_type,
                    range: (start_pos, end_pos),
                }],
            };

            snippet_slices.push(slice);
        }

        let snippet = Snippet {
            title: snippet_title,
            footer: vec![],
            slices: snippet_slices,
        };

        let renderer = create_diagnostics_renderer();
        tracing::error!("{}\n____\n", renderer.render(snippet));
    }

    fn get_title_label(diagnostics: &Diagnostic, label: &mut String) {
        label.clear();
        if let Some(reason) = diagnostics.reason() {
            label.push_str(reason.description());
        }
    }

    fn diagnostic_level_to_annotation_type(level: Level) -> AnnotationType {
        match level {
            Level::Info => AnnotationType::Info,
            Level::Warning => AnnotationType::Warning,
            Level::Error => AnnotationType::Error,
        }
    }
}

fn construct_slice(labels: Vec<&Label>) -> Slice {
    debug_assert!(
        !labels.is_empty(),
        "To construct slices, at least one label must be provided."
    );

    debug_assert!(
        labels.iter().all(|label| label.is_in_source()),
        "Slices can be constructed only for labels that are related to a place in source code."
    );

    debug_assert!(
        HashSet::<&str>::from_iter(labels.iter().map(|label| label.source_path().unwrap().as_str())).len() == 1,
        "Slices can be constructed only for labels that are related to places in the same source code."
    );

    let source_file = labels[0].source_path().map(|path| path.as_str());
    let source_code = labels[0].span().input();

    // Joint span of the code snippet that covers all the labels.
    let span = Span::join_all(labels.iter().map(|label| label.span().clone()));

    let (source, line_start, shift_in_bytes) = construct_code_snippet(&span, source_code);

    let mut annotations = vec![];

    for message in labels {
        annotations.push(SourceAnnotation {
            label: message.text(),
            annotation_type: label_type_to_annotation_type(message.label_type()),
            range: get_annotation_range(message.span(), source_code, shift_in_bytes),
        });
    }

    return Slice {
        source,
        line_start,
        origin: source_file,
        fold: true,
        annotations,
    };

    fn get_annotation_range(
        span: &Span,
        source_code: &str,
        shift_in_bytes: usize,
    ) -> (usize, usize) {
        let mut start_pos = span.start();
        let mut end_pos = span.end();

        let start_ix_bytes = start_pos - std::cmp::min(shift_in_bytes, start_pos);
        let end_ix_bytes = end_pos - std::cmp::min(shift_in_bytes, end_pos);

        // We want the start_pos and end_pos in terms of chars and not bytes, so translate.
        start_pos = source_code[shift_in_bytes..(shift_in_bytes + start_ix_bytes)]
            .chars()
            .count();
        end_pos = source_code[shift_in_bytes..(shift_in_bytes + end_ix_bytes)]
            .chars()
            .count();

        (start_pos, end_pos)
    }
}

fn label_type_to_annotation_type(label_type: LabelType) -> AnnotationType {
    match label_type {
        LabelType::Info => AnnotationType::Info,
        LabelType::Help => AnnotationType::Help,
        LabelType::Warning => AnnotationType::Warning,
        LabelType::Error => AnnotationType::Error,
    }
}

/// Given the overall span to be shown in the code snippet, determines how much of the input source
/// to show in the snippet.
///
/// Returns the source to be shown, the line start, and the offset of the snippet in bytes relative
/// to the beginning of the input code.
///
/// The library we use doesn't handle auto-windowing and line numbers, so we must manually
/// calculate the line numbers and match them up with the input window. It is a bit fiddly.
fn construct_code_snippet<'a>(span: &Span, input: &'a str) -> (&'a str, usize, usize) {
    // how many lines to prepend or append to the highlighted region in the window
    const NUM_LINES_BUFFER: usize = 2;

    let LineColRange { start, end } = span.line_col_one_index();

    let total_lines_in_input = input.chars().filter(|x| *x == '\n').count();
    debug_assert!(end.line >= start.line);
    let total_lines_of_highlight = end.line - start.line;
    debug_assert!(total_lines_in_input >= total_lines_of_highlight);

    let mut current_line = 0;
    let mut lines_to_start_of_snippet = 0;
    let mut calculated_start_ix = None;
    let mut calculated_end_ix = None;
    let mut pos = 0;
    for character in input.chars() {
        if character == '\n' {
            current_line += 1
        }

        if current_line + NUM_LINES_BUFFER >= start.line && calculated_start_ix.is_none() {
            calculated_start_ix = Some(pos);
            lines_to_start_of_snippet = current_line;
        }

        if current_line >= end.line + NUM_LINES_BUFFER && calculated_end_ix.is_none() {
            calculated_end_ix = Some(pos);
        }

        if calculated_start_ix.is_some() && calculated_end_ix.is_some() {
            break;
        }
        pos += character.len_utf8();
    }
    let calculated_start_ix = calculated_start_ix.unwrap_or(0);
    let calculated_end_ix = calculated_end_ix.unwrap_or(input.len());

    (
        &input[calculated_start_ix..calculated_end_ix],
        lines_to_start_of_snippet,
        calculated_start_ix,
    )
}

// TODO: Remove once "old-style" diagnostic is fully replaced with new one and the backward
//       compatibility is no longer needed.
/// Given a start and an end position and an input, determine how much of a window to show in the
/// error.
/// Mutates the start and end indexes to be in line with the new slice length.
///
/// The library we use doesn't handle auto-windowing and line numbers, so we must manually
/// calculate the line numbers and match them up with the input window. It is a bit fiddly.
fn construct_window<'a>(
    start: &mut LineCol,
    end: LineCol,
    start_ix: &mut usize,
    end_ix: &mut usize,
    input: &'a str,
) -> &'a str {
    // how many lines to prepend or append to the highlighted region in the window
    const NUM_LINES_BUFFER: usize = 2;

    let total_lines_in_input = input.chars().filter(|x| *x == '\n').count();
    debug_assert!(end.line >= start.line);
    let total_lines_of_highlight = end.line - start.line;
    debug_assert!(total_lines_in_input >= total_lines_of_highlight);

    let mut current_line = 1usize;

    let mut chars = input.char_indices().map(|(char_offset, character)| {
        let r = (current_line, char_offset);
        if character == '\n' {
            current_line += 1;
        }
        r
    });

    // Find the first char of the first line
    let first_char = chars
        .by_ref()
        .find(|(current_line, _)| current_line + NUM_LINES_BUFFER >= start.line);

    // Find the last char of the last line
    let last_char = chars
        .by_ref()
        .find(|(current_line, _)| *current_line > end.line + NUM_LINES_BUFFER)
        .map(|x| x.1);

    // this releases the borrow of `current_line`
    drop(chars);

    let (first_char_line, first_char_offset, last_char_offset) = match (first_char, last_char) {
        // has first and last
        (Some((first_char_line, first_char_offset)), Some(last_char_offset)) => {
            (first_char_line, first_char_offset, last_char_offset)
        }
        // has first and no last
        (Some((first_char_line, first_char_offset)), None) => {
            (first_char_line, first_char_offset, input.len())
        }
        // others
        _ => (current_line, input.len(), input.len()),
    };

    // adjust indices to be inside the returned window
    start.line = first_char_line;
    *start_ix = start_ix.saturating_sub(first_char_offset);
    *end_ix = end_ix.saturating_sub(first_char_offset);

    &input[first_char_offset..last_char_offset]
}

#[test]
fn ok_construct_window() {
    fn t(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        start_char: usize,
        end_char: usize,
        input: &str,
    ) -> (usize, usize, &str) {
        let mut s = LineCol {
            line: start_line,
            col: start_col,
        };
        let mut start = start_char;
        let mut end = end_char;
        let r = construct_window(
            &mut s,
            LineCol {
                line: end_line,
                col: end_col,
            },
            &mut start,
            &mut end,
            input,
        );
        (start, end, r)
    }

    // Invalid Empty file
    assert_eq!(t(0, 0, 0, 0, 0, 0, ""), (0, 0, ""));

    // Valid Empty File
    assert_eq!(t(1, 1, 1, 1, 0, 0, ""), (0, 0, ""));

    // One line, error after the last char
    assert_eq!(t(1, 7, 1, 7, 6, 6, "script"), (6, 6, "script"));

    //                       01 23 45 67 89 AB CD E
    let eight_lines = "1\n2\n3\n4\n5\n6\n7\n8";

    assert_eq!(t(1, 1, 1, 1, 0, 1, eight_lines), (0, 1, "1\n2\n3\n"));
    assert_eq!(t(2, 1, 2, 1, 2, 3, eight_lines), (2, 3, "1\n2\n3\n4\n"));
    assert_eq!(t(3, 1, 3, 1, 4, 5, eight_lines), (4, 5, "1\n2\n3\n4\n5\n"));
    assert_eq!(t(4, 1, 4, 1, 6, 7, eight_lines), (4, 5, "2\n3\n4\n5\n6\n"));
    assert_eq!(t(5, 1, 5, 1, 8, 9, eight_lines), (4, 5, "3\n4\n5\n6\n7\n"));
    assert_eq!(t(6, 1, 6, 1, 10, 11, eight_lines), (4, 5, "4\n5\n6\n7\n8"));
    assert_eq!(t(7, 1, 7, 1, 12, 13, eight_lines), (4, 5, "5\n6\n7\n8"));
    assert_eq!(t(8, 1, 8, 1, 14, 15, eight_lines), (4, 5, "6\n7\n8"));

    // Invalid lines
    assert_eq!(t(9, 1, 9, 1, 14, 15, eight_lines), (2, 3, "7\n8"));
    assert_eq!(t(10, 1, 10, 1, 14, 15, eight_lines), (0, 1, "8"));
    assert_eq!(t(11, 1, 11, 1, 14, 15, eight_lines), (0, 0, ""));
}
