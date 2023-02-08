use crate::{
    config::OnEnterConfig,
    core::{document::TextDocument, session::Session},
};
use std::sync::Arc;
use tower_lsp::{
    lsp_types::{
        DidChangeTextDocumentParams, DocumentChanges, OneOf,
        OptionalVersionedTextDocumentIdentifier, Position, Range, TextDocumentEdit, TextEdit, Url,
        WorkspaceEdit,
    },
    Client,
};

const NEWLINE: &str = "\n";
const COMMENT_START: &str = "//";
const DOC_COMMENT_START: &str = "///";

/// If the change was an enter keypress or pasting multiple lines in a comment, it prefixes the line(s)
/// with the appropriate comment start pattern (// or ///).
pub(crate) async fn on_enter(
    config: &OnEnterConfig,
    client: &Client,
    session: &Arc<Session>,
    temp_uri: &Url,
    params: &DidChangeTextDocumentParams,
) {
    if !(params.content_changes[0].text.contains(NEWLINE)) {
        return;
    }

    let mut workspace_edit = None;
    let text_document = session
        .get_text_document(temp_uri)
        .expect("could not get text document");

    if config.continue_doc_comments.unwrap_or(false) {
        workspace_edit = get_comment_workspace_edit(DOC_COMMENT_START, params, &text_document);
    }

    if config.continue_comments.unwrap_or(false) && workspace_edit.is_none() {
        workspace_edit = get_comment_workspace_edit(COMMENT_START, params, &text_document);
    }

    // Apply any edits.
    if let Some(edit) = workspace_edit {
        if let Err(err) = client.apply_edit(edit).await {
            tracing::error!("on_enter failed to apply edit: {}", err);
        }
    }
}

fn get_comment_workspace_edit(
    start_pattern: &str,
    change_params: &DidChangeTextDocumentParams,
    text_document: &TextDocument,
) -> Option<WorkspaceEdit> {
    let range = change_params.content_changes[0]
        .range
        .expect("change is missing range");
    let line = text_document.get_line(range.start.line as usize);
    if line.trim().starts_with(start_pattern) {
        let uri = change_params.text_document.uri.clone();
        let text = change_params.content_changes[0].text.clone();

        let indentation = &line[..line.find(start_pattern).unwrap_or(0)];
        let mut edits = vec![];

        // To support pasting multiple lines in a comment, we need to add the comment start pattern after each newline,
        // except the last one.
        let lines: Vec<_> = text.split(NEWLINE).collect();
        lines.iter().enumerate().for_each(|(i, _)| {
            if i < lines.len() - 1 {
                let position =
                    Position::new(range.start.line + (i as u32) + 1, indentation.len() as u32);
                edits.push(OneOf::Left(TextEdit {
                    new_text: format!("{start_pattern} "),
                    range: Range::new(position, position),
                }));
            }
        });
        let edit = TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                // Use the original uri to make updates, not the temporary one from the session.
                uri,
                version: None,
            },
            edits,
        };
        Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(vec![edit])),
            ..Default::default()
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_lsp_test_utils::get_absolute_path;
    use tower_lsp::lsp_types::{
        AnnotatedTextEdit, TextDocumentContentChangeEvent, VersionedTextDocumentIdentifier,
    };

    fn assert_text_edit(
        actual: &OneOf<TextEdit, AnnotatedTextEdit>,
        new_text: String,
        line: u32,
        character: u32,
    ) {
        match actual {
            OneOf::Left(edit) => {
                let position = Position { line, character };
                let expected = TextEdit {
                    new_text,
                    range: Range {
                        start: position,
                        end: position,
                    },
                };
                assert_eq!(*edit, expected);
            }
            OneOf::Right(_) => panic!("expected left"),
        }
    }

    #[test]
    fn get_comment_workspace_edit_double_slash_indented() {
        let path = get_absolute_path("sway-lsp/tests/fixtures/diagnostics/dead_code/src/main.sw");
        let uri = Url::from_file_path(path.clone()).unwrap();
        let text_document =
            TextDocument::build_from_path(path.as_str()).expect("failed to build document");
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: 1 },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 47,
                        character: 34,
                    },
                    end: Position {
                        line: 47,
                        character: 34,
                    },
                }),
                range_length: Some(0),
                text: "\n    ".to_string(),
            }],
        };

        let result = get_comment_workspace_edit(COMMENT_START, &params, &text_document)
            .expect("workspace edit");
        let changes = result.document_changes.expect("document changes");
        let edits = match changes {
            DocumentChanges::Edits(edits) => edits,
            DocumentChanges::Operations(_) => panic!("expected edits"),
        };

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].edits.len(), 1);
        assert_text_edit(&edits[0].edits[0], "// ".to_string(), 48, 4);
    }

    #[test]
    fn get_comment_workspace_edit_triple_slash_paste() {
        let path = get_absolute_path("sway-lsp/tests/fixtures/diagnostics/dead_code/src/main.sw");
        let uri = Url::from_file_path(path.clone()).unwrap();
        let text_document =
            TextDocument::build_from_path(path.as_str()).expect("failed to build document");
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: 1 },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 41,
                        character: 4,
                    },
                    end: Position {
                        line: 41,
                        character: 34,
                    },
                }),
                range_length: Some(30),
                text: "fn not_used2(input: u64) -> u64 {\n    return input + 1;\n}".to_string(),
            }],
        };

        let result = get_comment_workspace_edit(DOC_COMMENT_START, &params, &text_document)
            .expect("workspace edit");
        let changes = result.document_changes.expect("document changes");
        let edits = match changes {
            DocumentChanges::Edits(edits) => edits,
            DocumentChanges::Operations(_) => panic!("expected edits"),
        };

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].edits.len(), 2);
        assert_text_edit(&edits[0].edits[0], "/// ".to_string(), 42, 0);
        assert_text_edit(&edits[0].edits[1], "/// ".to_string(), 43, 0);
    }
}
