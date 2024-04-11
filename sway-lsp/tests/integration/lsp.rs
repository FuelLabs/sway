//! This file contains the methods used for simulating LSP json-rpc notifications and requests.
//! The methods are used to build and send requests and notifications to the LSP service
//! and assert the expected responses.

use crate::{GotoDefinition, HoverDocumentation, Rename};
use assert_json_diff::assert_json_eq;
use regex::Regex;
use serde_json::json;
use std::{borrow::Cow, path::Path};
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

pub(crate) async fn initialize_request(service: &mut LspService<ServerState>) -> Request {
    let params = json!({ "capabilities": sway_lsp::server_capabilities() });
    let initialize = build_request_with_id("initialize", params, 1);
    let response = call_request(service, initialize.clone()).await;
    let expected = Response::from_ok(
        1.into(),
        json!({ "capabilities": sway_lsp::server_capabilities() }),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    initialize
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

    let response = request::handle_show_ast(server, params).await;
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

    let response = request::handle_visualize(server, params).unwrap().unwrap();
    let re = Regex::new(r#"digraph \{
    0 \[ label = "core" shape = box URL = "vscode://file/[[:ascii:]]+/sway-lib-core/Forc.toml"\]
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

pub(crate) async fn semantic_tokens_request(server: &ServerState, uri: &Url) {
    let params = SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = request::handle_semantic_tokens_full(server, params)
        .await
        .unwrap();
    if let Some(SemanticTokensResult::Tokens(tokens)) = response {
        assert!(!tokens.data.is_empty());
    }
}

pub(crate) async fn document_symbol_request(server: &ServerState, uri: &Url) {
    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    };
    let response = request::handle_document_symbol(server, params)
        .await
        .unwrap();
    if let Some(DocumentSymbolResponse::Flat(res)) = response {
        assert!(!res.is_empty());
    }
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

pub(crate) async fn completion_request(server: &ServerState, uri: &Url) {
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 19,
                character: 8,
            },
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: Some(CompletionContext {
            trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
            trigger_character: Some(".".to_string()),
        }),
    };
    let res = request::handle_completion(server, params).await.unwrap();
    let expected = CompletionResponse::Array(vec![
        CompletionItem {
            label: "a".to_string(),
            kind: Some(CompletionItemKind::FIELD),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: Some("bool".to_string()),
            }),
            ..Default::default()
        },
        CompletionItem {
            label: "get(…)".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: Some("fn(self, MyStruct) -> MyStruct".to_string()),
            }),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range: Range {
                    start: Position {
                        line: 19,
                        character: 8,
                    },
                    end: Position {
                        line: 19,
                        character: 8,
                    },
                },
                new_text: "get(foo)".to_string(),
            })),
            ..Default::default()
        },
    ]);
    assert_eq!(expected, res.unwrap());
}

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
    let res = request::handle_goto_definition(server, params.clone())
        .await
        .unwrap();
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
    let res = request::handle_hover(server, params.clone()).await.unwrap();
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
    request::handle_prepare_rename(server, params)
        .await
        .unwrap()
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
    let worspace_edit = request::handle_rename(server, params).await.unwrap();
    worspace_edit.unwrap()
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
