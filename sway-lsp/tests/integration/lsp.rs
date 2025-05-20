//! This file contains the methods used for simulating LSP json-rpc notifications and requests.
//! The methods are used to build and send requests and notifications to the LSP service
//! and assert the expected responses.

use crate::{GotoDefinition, HoverDocumentation, Rename};
use assert_json_diff::assert_json_eq;
use forc_pkg::manifest::GenericManifestFile;
use forc_pkg::manifest::ManifestFile;
use regex::Regex;
use serde_json::json;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use sway_lsp::{
    handlers::request,
    lsp_ext::{ShowAstParams, VisualizeParams},
    server_state::ServerState,
};
use sway_utils::PerformanceData;
use tower::{Service, ServiceExt};
use tower_lsp::{
    jsonrpc::{Id, Request, Response},
    lsp_types::*,
    ExitedError, LspService,
};

pub(crate) fn build_request_with_id(
    method: impl Into<Cow<'static, str>>,
    params: serde_json::Value,
    id: impl Into<Id>,
) -> Request {
    Request::build(method).params(params).id(id).finish()
}

pub(crate) async fn call_request(
    service: &mut LspService<ServerState>,
    req: Request,
) -> Result<Option<Response>, ExitedError> {
    service.ready().await?.call(req).await
}

pub(crate) fn client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        workspace: Some(WorkspaceClientCapabilities {
            workspace_folders: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(crate) async fn initialize_request(
    service: &mut LspService<ServerState>,
    entry_point: &Path,
) -> Request {
    let search_dir = entry_point.parent().unwrap_or_else(|| Path::new(""));
    let project_root_path_for_uri: PathBuf = match ManifestFile::from_dir(search_dir) {
        Ok(manifest_file) => {
            // Found a Forc.toml, use its directory
            manifest_file.dir().to_path_buf()
        }
        Err(_) => {
            // Forc.toml not found, assume search_dir is the intended project root for this test fixture.
            // This is common for minimal test cases that might only have a src/main.sw
            search_dir.to_path_buf()
        }
    };

    let root_uri = Url::from_directory_path(&project_root_path_for_uri).unwrap_or_else(|_| {
        panic!(
            "Failed to create directory URL from project root: {:?}",
            project_root_path_for_uri
        )
    });

    // Construct the InitializeParams using the defined client_capabilities
    let params = json!({
        "processId": Option::<u32>::None,
        "rootUri": Some(root_uri),
        "capabilities": client_capabilities(),
        "initializationOptions": Option::<serde_json::Value>::None,
    });

    let initialize_request = build_request_with_id("initialize", params, 1); // Renamed for clarity
    let response = call_request(service, initialize_request.clone()).await;

    let expected_initialize_result = json!({ "capabilities": sway_lsp::server_capabilities() });
    let expected_response = Response::from_ok(1.into(), expected_initialize_result);

    assert!(
        response.is_ok(),
        "Initialize request failed: {:?}",
        response.err()
    );
    assert_json_eq!(expected_response, response.ok().unwrap());

    initialize_request
}

pub(crate) async fn initialized_notification(service: &mut LspService<ServerState>) {
    let initialized = Request::build("initialized").finish();
    let response = call_request(service, initialized).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn shutdown_request(service: &mut LspService<ServerState>) -> Request {
    let shutdown = Request::build("shutdown").id(1).finish();
    let response = call_request(service, shutdown.clone()).await;
    let expected = Response::from_ok(1.into(), json!(null));
    assert_json_eq!(expected, response.ok().unwrap());
    shutdown
}

pub(crate) async fn exit_notification(service: &mut LspService<ServerState>) {
    let exit = Request::build("exit").finish();
    let response = call_request(service, exit.clone()).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn did_change_watched_files_notification(
    service: &mut LspService<ServerState>,
    params: DidChangeWatchedFilesParams,
) {
    let params: serde_json::value::Value = serde_json::to_value(params).unwrap();
    let did_change_watched_files = Request::build("workspace/didChangeWatchedFiles")
        .params(params)
        .finish();
    let response = call_request(service, did_change_watched_files).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn did_open_notification(
    service: &mut LspService<ServerState>,
    uri: &Url,
    text: &str,
) {
    let params = json!({
        "textDocument": {
            "uri": uri,
            "languageId": "sway",
            "version": 1,
            "text": text,
        },
    });

    let did_open = Request::build("textDocument/didOpen")
        .params(params)
        .finish();
    let response = call_request(service, did_open).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn did_change_request(
    service: &mut LspService<ServerState>,
    uri: &Url,
    version: i32,
    params: Option<DidChangeTextDocumentParams>,
) -> Request {
    let params = params.unwrap_or_else(|| {
        create_did_change_params(
            uri,
            version,
            Position {
                line: 1,
                character: 0,
            },
            Position {
                line: 1,
                character: 0,
            },
            0,
        )
    });
    let params: serde_json::value::Value = serde_json::to_value(params).unwrap();
    let did_change = Request::build("textDocument/didChange")
        .params(params)
        .finish();
    let response = call_request(service, did_change.clone()).await;
    // make sure to set is_compiling to true so the wait_for_parsing method can properly synchnonize
    service
        .inner()
        .is_compiling
        .store(true, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(response, Ok(None));
    did_change
}

/// Simulates a keypress at the current cursor position
/// 66% chance of enter keypress
/// 33% chance of backspace keypress
pub fn simulate_keypress(
    uri: &Url,
    version: i32,
    cursor_line: &mut u32,
) -> DidChangeTextDocumentParams {
    if rand::random::<u64>() % 3 < 2 {
        // enter keypress at current cursor line
        *cursor_line += 1;
        create_did_change_params(
            uri,
            version,
            Position {
                line: *cursor_line - 1,
                character: 0,
            },
            Position {
                line: *cursor_line - 1,
                character: 0,
            },
            0,
        )
    } else {
        // backspace keypress at current cursor line
        if *cursor_line > 1 {
            *cursor_line -= 1;
        }
        create_did_change_params(
            uri,
            version,
            Position {
                line: *cursor_line,
                character: 0,
            },
            Position {
                line: *cursor_line + 1,
                character: 0,
            },
            1,
        )
    }
}

pub(crate) async fn show_ast_request(
    server: &ServerState,
    uri: &Url,
    ast_kind: &str,
    save_path: Option<Url>,
) {
    // The path where the AST will be written to.
    // If no path is provided, the default path is "/tmp"
    let save_path = match save_path {
        Some(path) => path,
        None => Url::from_file_path(Path::new("/tmp")).unwrap(),
    };
    let params = ShowAstParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        ast_kind: ast_kind.to_string(),
        save_path: save_path.clone(),
    };

    let response = request::handle_show_ast(server, &params);
    let expected = TextDocumentIdentifier {
        uri: Url::parse(&format!("{save_path}/{ast_kind}.rs")).unwrap(),
    };
    assert_eq!(expected, response.unwrap().unwrap());
}

pub(crate) async fn visualize_request(server: &ServerState, uri: &Url, graph_kind: &str) {
    let params = VisualizeParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        graph_kind: graph_kind.to_string(),
    };

    let response = request::handle_visualize(server, &params).unwrap().unwrap();
    let re = Regex::new(r#"digraph \{
    0 \[ label = "std" shape = box URL = "vscode://file/[[:ascii:]]+/sway-lib-std/Forc.toml"\]
    1 \[ label = "struct_field_access" shape = box URL = "vscode://file/[[:ascii:]]+/struct_field_access/Forc.toml"\]
    1 -> 0 \[ \]
\}
"#).unwrap();
    assert!(!re.find(response.as_str()).unwrap().is_empty());
}

pub(crate) async fn metrics_request(
    service: &mut LspService<ServerState>,
    uri: &Url,
) -> Vec<(String, PerformanceData)> {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
    });
    let request = build_request_with_id("sway/metrics", params, 1);
    let result = call_request(service, request.clone())
        .await
        .unwrap()
        .unwrap();
    let value = result.result().unwrap().as_array();
    let mut res = vec![];
    for v in value.unwrap().iter() {
        let path = v.get(0).unwrap().as_str().unwrap();
        let metric = serde_json::from_value(v.get(1).unwrap().clone()).unwrap();
        res.push((path.to_string(), metric));
    }
    res
}

pub(crate) async fn get_semantic_tokens_full(server: &ServerState, uri: &Url) -> SemanticTokens {
    let params = SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = request::handle_semantic_tokens_full(server, params)
        .await
        .unwrap();
    if let Some(SemanticTokensResult::Tokens(tokens)) = response {
        tokens
    } else {
        panic!("Expected semantic tokens response");
    }
}

pub(crate) async fn semantic_tokens_request(server: &ServerState, uri: &Url) {
    let tokens = get_semantic_tokens_full(server, uri).await;
    assert!(!tokens.data.is_empty());
}

pub(crate) async fn get_nested_document_symbols(
    server: &ServerState,
    uri: &Url,
) -> Vec<DocumentSymbol> {
    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    if let Some(DocumentSymbolResponse::Nested(symbols)) =
        request::handle_document_symbol(server, params)
            .await
            .unwrap()
    {
        symbols
    } else {
        panic!("Expected nested document symbols response");
    }
}

pub(crate) async fn document_symbols_request(server: &ServerState, uri: &Url) {
    let symbols = get_nested_document_symbols(server, uri).await;
    // Check for enum with its variants
    let enum_symbol = symbols
        .iter()
        .find(|s| s.name == "NumberOrString")
        .expect("Should find NumberOrString enum");
    assert_eq!(enum_symbol.kind, SymbolKind::ENUM);
    let variants = enum_symbol
        .children
        .as_ref()
        .expect("Enum should have variants");
    assert_eq!(variants.len(), 2);
    assert!(variants.iter().any(|v| v.name == "Number"));
    assert!(variants.iter().any(|v| v.name == "String"));

    // Check for struct with its fields
    let struct_symbol = symbols
        .iter()
        .find(|s| s.name == "Data")
        .expect("Should find Data struct");
    assert_eq!(struct_symbol.kind, SymbolKind::STRUCT);
    let fields = struct_symbol
        .children
        .as_ref()
        .expect("Struct should have fields");
    assert_eq!(fields.len(), 2);
    assert!(fields
        .iter()
        .any(|f| f.name == "value" && f.detail.as_deref() == Some("NumberOrString")));
    assert!(fields
        .iter()
        .any(|f| f.name == "address" && f.detail.as_deref() == Some("u64")));

    // Check for impl with nested function and variable
    let impl_symbol = symbols
        .iter()
        .find(|s| s.name == "impl FooABI for Contract")
        .expect("Should find impl block");
    let impl_fns = impl_symbol
        .children
        .as_ref()
        .expect("Impl should have functions");
    let main_fn = impl_fns
        .iter()
        .find(|f| f.name == "main")
        .expect("Should find main function");
    let vars = main_fn
        .children
        .as_ref()
        .expect("Function should have variables");
    assert!(vars
        .iter()
        .any(|v| v.name == "_data" && v.detail.as_deref() == Some("Data")));
}

pub(crate) async fn format_request(server: &ServerState, uri: &Url) {
    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: Default::default(),
    };
    let response = request::handle_formatting(server, params).await.unwrap();
    assert!(!response.unwrap().is_empty());
}

pub(crate) async fn highlight_request(server: &ServerState, uri: &Url) {
    let params = DocumentHighlightParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 45,
                character: 37,
            },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = request::handle_document_highlight(server, params)
        .await
        .unwrap();
    let expected = vec![
        DocumentHighlight {
            range: Range {
                start: Position {
                    line: 10,
                    character: 4,
                },
                end: Position {
                    line: 10,
                    character: 10,
                },
            },
            kind: None,
        },
        DocumentHighlight {
            range: Range {
                start: Position {
                    line: 45,
                    character: 35,
                },
                end: Position {
                    line: 45,
                    character: 41,
                },
            },
            kind: None,
        },
    ];
    assert_eq!(expected, response.unwrap());
}

pub(crate) async fn references_request(server: &ServerState, uri: &Url) {
    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 15,
                character: 22,
            },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: ReferenceContext {
            include_declaration: false,
        },
    };

    let create_location = |line: u32, start_char: u32, end_char: u32| -> Location {
        Location {
            uri: uri.clone(),
            range: Range {
                start: Position {
                    line,
                    character: start_char,
                },
                end: Position {
                    line,
                    character: end_char,
                },
            },
        }
    };

    let mut response = request::handle_references(server, params)
        .await
        .unwrap()
        .unwrap();

    let mut expected = vec![
        create_location(12, 7, 11),
        create_location(15, 21, 25),
        create_location(15, 14, 18),
        create_location(13, 13, 17),
        create_location(3, 5, 9),
        create_location(14, 8, 12),
    ];
    response.sort_by(|a, b| a.range.start.cmp(&b.range.start));
    expected.sort_by(|a, b| a.range.start.cmp(&b.range.start));
    assert_eq!(expected, response);
}

pub(crate) async fn code_lens_empty_request(server: &ServerState, uri: &Url) {
    let params = CodeLensParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = request::handle_code_lens(server, params).await.unwrap();
    assert_eq!(response.unwrap().len(), 0);
}

pub(crate) async fn code_lens_request(server: &ServerState, uri: &Url) {
    let params = CodeLensParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = request::handle_code_lens(server, params).await.unwrap();
    let expected = vec![
        CodeLens {
            range: Range {
                start: Position {
                    line: 2,
                    character: 3,
                },
                end: Position {
                    line: 2,
                    character: 7,
                },
            },
            command: Some(Command {
                title: "▶︎ Run".to_string(),
                command: "sway.runScript".to_string(),
                arguments: None,
            }),
            data: None,
        },
        CodeLens {
            range: Range {
                start: Position {
                    line: 6,
                    character: 0,
                },
                end: Position {
                    line: 6,
                    character: 7,
                },
            },
            command: Some(Command {
                title: "▶︎ Run Test".to_string(),
                command: "sway.runTests".to_string(),
                arguments: Some(vec![json!({
                    "name": "test_foo"
                })]),
            }),
            data: None,
        },
        CodeLens {
            range: Range {
                start: Position {
                    line: 11,
                    character: 0,
                },
                end: Position {
                    line: 11,
                    character: 7,
                },
            },
            command: Some(Command {
                title: "▶︎ Run Test".to_string(),
                command: "sway.runTests".to_string(),
                arguments: Some(vec![json!({
                    "name": "test_bar"
                })]),
            }),
            data: None,
        },
    ];
    assert_eq!(expected, response.unwrap());
}

// pub(crate) async fn completion_request(server: &ServerState, uri: &Url) {
//     let params = CompletionParams {
//         text_document_position: TextDocumentPositionParams {
//             text_document: TextDocumentIdentifier { uri: uri.clone() },
//             position: Position {
//                 line: 19,
//                 character: 8,
//             },
//         },
//         work_done_progress_params: Default::default(),
//         partial_result_params: Default::default(),
//         context: Some(CompletionContext {
//             trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
//             trigger_character: Some(".".to_string()),
//         }),
//     };
//     let res = request::handle_completion(server, params).await.unwrap();
//     let expected = CompletionResponse::Array(vec![
//         CompletionItem {
//             label: "a".to_string(),
//             kind: Some(CompletionItemKind::FIELD),
//             label_details: Some(CompletionItemLabelDetails {
//                 detail: None,
//                 description: Some("bool".to_string()),
//             }),
//             ..Default::default()
//         },
//         CompletionItem {
//             label: "get(…)".to_string(),
//             kind: Some(CompletionItemKind::METHOD),
//             label_details: Some(CompletionItemLabelDetails {
//                 detail: None,
//                 description: Some("fn(self, MyStruct) -> MyStruct".to_string()),
//             }),
//             text_edit: Some(CompletionTextEdit::Edit(TextEdit {
//                 range: Range {
//                     start: Position {
//                         line: 19,
//                         character: 8,
//                     },
//                     end: Position {
//                         line: 19,
//                         character: 8,
//                     },
//                 },
//                 new_text: "get(foo)".to_string(),
//             })),
//             ..Default::default()
//         },
//     ]);
//     assert_eq!(expected, res.unwrap());
// }

pub(crate) async fn definition_check<'a>(server: &ServerState, go_to: &'a GotoDefinition<'a>) {
    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: go_to.req_uri.clone(),
            },
            position: Position {
                line: go_to.req_line,
                character: go_to.req_char,
            },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let res = request::handle_goto_definition(server, params.clone()).unwrap();
    let unwrapped_response = res.as_ref().unwrap_or_else(|| {
        panic!(
            "Failed to deserialize response: {:?} input: {:#?}",
            res.clone(),
            params.clone(),
        );
    });
    if let GotoDefinitionResponse::Scalar(response) = unwrapped_response {
        let uri = response.uri.as_str();
        let range = json!({
            "end": {
                "character": go_to.def_end_char,
                "line": go_to.def_line,
            },
            "start": {
                "character": go_to.def_start_char,
                "line": go_to.def_line,
            }
        });
        assert_json_eq!(response.range, range);
        assert!(
            uri.ends_with(go_to.def_path),
            "{} doesn't end with {}",
            uri,
            go_to.def_path,
        );
    } else {
        panic!(
            "Expected GotoDefinitionResponse::Scalar with input {:#?}, got {:?}",
            params.clone(),
            res.clone(),
        );
    }
}

pub(crate) async fn definition_check_with_req_offset(
    server: &ServerState,
    go_to: &mut GotoDefinition<'_>,
    req_line: u32,
    req_char: u32,
) {
    go_to.req_line = req_line;
    go_to.req_char = req_char;
    definition_check(server, go_to).await;
}

pub(crate) async fn hover_request<'a>(
    server: &ServerState,
    hover_docs: &'a HoverDocumentation<'a>,
) {
    let params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: hover_docs.req_uri.clone(),
            },
            position: Position {
                line: hover_docs.req_line,
                character: hover_docs.req_char,
            },
        },
        work_done_progress_params: Default::default(),
    };
    let res = request::handle_hover(server, params.clone()).unwrap();
    let unwrapped_response = res.as_ref().unwrap_or_else(|| {
        panic!(
            "Failed to deserialize hover: {:?} input: {:#?}",
            res.clone(),
            params.clone(),
        );
    });
    if let HoverContents::Markup(markup_content) = &unwrapped_response.contents {
        hover_docs
            .documentation
            .iter()
            .for_each(|text| assert!(markup_content.value.contains(text)));
    } else {
        panic!(
            "Expected HoverContents::Markup with input {:#?}, got {:?}",
            res.clone(),
            params.clone(),
        );
    }
}

pub(crate) async fn prepare_rename_request<'a>(
    server: &ServerState,
    rename: &'a Rename<'a>,
) -> Option<PrepareRenameResponse> {
    let params = TextDocumentPositionParams {
        text_document: TextDocumentIdentifier {
            uri: rename.req_uri.clone(),
        },
        position: Position {
            line: rename.req_line,
            character: rename.req_char,
        },
    };
    request::handle_prepare_rename(server, params).unwrap()
}

pub(crate) async fn rename_request<'a>(
    server: &ServerState,
    rename: &'a Rename<'a>,
) -> WorkspaceEdit {
    let params = RenameParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: rename.req_uri.clone(),
            },
            position: Position {
                line: rename.req_line,
                character: rename.req_char,
            },
        },
        new_name: rename.new_name.to_string(),
        work_done_progress_params: Default::default(),
    };
    let workspace_edit = request::handle_rename(server, params).unwrap();
    workspace_edit.unwrap()
}

pub fn create_did_change_params(
    uri: &Url,
    version: i32,
    start: Position,
    end: Position,
    range_length: u32,
) -> DidChangeTextDocumentParams {
    DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: Some(Range { start, end }),
            range_length: Some(range_length),
            text: "\n".into(),
        }],
    }
}

#[allow(dead_code)]
pub(crate) fn range_from_start_and_end_line(start_line: u32, end_line: u32) -> Range {
    Range {
        start: Position {
            line: start_line,
            character: 0,
        },
        end: Position {
            line: end_line,
            character: 0,
        },
    }
}

pub(crate) async fn get_inlay_hints_for_range(
    server: &ServerState,
    uri: &Url,
    range: Range,
) -> Vec<InlayHint> {
    let params = InlayHintParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        range,
        work_done_progress_params: Default::default(),
    };
    request::handle_inlay_hints(server, params)
        .await
        .unwrap()
        .unwrap()
}

pub(crate) async fn inlay_hints_request(server: &ServerState, uri: &Url) -> Option<Vec<InlayHint>> {
    let range = Range {
        start: Position {
            line: 25,
            character: 0,
        },
        end: Position {
            line: 26,
            character: 1,
        },
    };
    let res = get_inlay_hints_for_range(server, uri, range).await;
    let expected = vec![
        InlayHint {
            position: Position {
                line: 25,
                character: 25,
            },
            label: InlayHintLabel::String("foo: ".to_string()),
            kind: Some(InlayHintKind::PARAMETER),
            text_edits: None,
            tooltip: None,
            padding_left: Some(false),
            padding_right: Some(false),
            data: None,
        },
        InlayHint {
            position: Position {
                line: 25,
                character: 28,
            },
            label: InlayHintLabel::String("bar: ".to_string()),
            kind: Some(InlayHintKind::PARAMETER),
            text_edits: None,
            tooltip: None,
            padding_left: Some(false),
            padding_right: Some(false),
            data: None,
        },
        InlayHint {
            position: Position {
                line: 25,
                character: 31,
            },
            label: InlayHintLabel::String("long_argument_name: ".to_string()),
            kind: Some(InlayHintKind::PARAMETER),
            text_edits: None,
            tooltip: None,
            padding_left: Some(false),
            padding_right: Some(false),
            data: None,
        },
        InlayHint {
            position: Position {
                line: 25,
                character: 10,
            },
            label: InlayHintLabel::String(": u64".to_string()),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: None,
            padding_left: Some(false),
            padding_right: Some(false),
            data: None,
        },
    ];

    assert!(
        compare_inlay_hint_vecs(&expected, &res),
        "InlayHint vectors are not equal.\nExpected:\n{:#?}\n\nActual:\n{:#?}",
        expected,
        res
    );
    Some(res)
}

// This is a helper function to compare two inlay hints. because PartialEq is not implemented for InlayHint
fn compare_inlay_hints(a: &InlayHint, b: &InlayHint) -> bool {
    a.position == b.position
        && compare_inlay_hint_labels(&a.label, &b.label)
        && a.kind == b.kind
        && a.text_edits == b.text_edits
        && compare_inlay_hint_tooltips(&a.tooltip, &b.tooltip)
        && a.padding_left == b.padding_left
        && a.padding_right == b.padding_right
        && a.data == b.data
}

fn compare_inlay_hint_vecs(a: &[InlayHint], b: &[InlayHint]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| compare_inlay_hints(a, b))
}

fn compare_inlay_hint_labels(a: &InlayHintLabel, b: &InlayHintLabel) -> bool {
    match (a, b) {
        (InlayHintLabel::String(a), InlayHintLabel::String(b)) => a == b,
        _ => false,
    }
}

fn compare_inlay_hint_tooltips(a: &Option<InlayHintTooltip>, b: &Option<InlayHintTooltip>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => match (a, b) {
            (InlayHintTooltip::String(a), InlayHintTooltip::String(b)) => a == b,
            (InlayHintTooltip::MarkupContent(a), InlayHintTooltip::MarkupContent(b)) => {
                a.kind == b.kind && a.value == b.value
            }
            _ => false,
        },
        _ => false,
    }
}
