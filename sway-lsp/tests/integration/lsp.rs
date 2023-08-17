//! This file contains the methods used for simulating LSP json-rpc notifications and requests.
//! The methods are used to build and send requests and notifications to the LSP service
//! and assert the expected responses.

use crate::{GotoDefinition, HoverDocumentation, Rename};
use assert_json_diff::assert_json_eq;
use serde_json::json;
use std::{borrow::Cow, path::Path};
use sway_lsp::{handlers::request, lsp_ext::ShowAstParams, server_state::ServerState};
use sway_lsp_test_utils::extract_result_array;
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
) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
            "version": 2
        },
        "contentChanges": [
            {
                "range": {
                    "start": {
                        "line": 1,
                        "character": 0
                    },
                    "end": {
                        "line": 1,
                        "character": 0
                    }
                },
                "rangeLength": 0,
                "text": "\n",
            }
        ]
    });
    let did_change = Request::build("textDocument/didChange")
        .params(params)
        .finish();
    let response = call_request(service, did_change.clone()).await;
    assert_eq!(response, Ok(None));
    did_change
}

pub(crate) async fn did_close_notification(service: &mut LspService<ServerState>) {
    let exit = Request::build("textDocument/didClose").finish();
    let response = call_request(service, exit.clone()).await;
    assert_eq!(response, Ok(None));
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

    let response = request::handle_show_ast(server, params);
    let expected = TextDocumentIdentifier {
        uri: Url::parse(&format!("{save_path}/{ast_kind}.rs")).unwrap(),
    };
    assert_eq!(expected, response.unwrap().unwrap());
}

pub(crate) async fn semantic_tokens_request(
    service: &mut LspService<ServerState>,
    uri: &Url,
) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
    });
    let semantic_tokens = build_request_with_id("textDocument/semanticTokens/full", params, 1);
    let _response = call_request(service, semantic_tokens.clone()).await;
    semantic_tokens
}

pub(crate) async fn document_symbol_request(
    service: &mut LspService<ServerState>,
    uri: &Url,
) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
    });
    let document_symbol = build_request_with_id("textDocument/documentSymbol", params, 1);
    let _response = call_request(service, document_symbol.clone()).await;
    document_symbol
}

pub(crate) async fn format_request(service: &mut LspService<ServerState>, uri: &Url) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
        "options": {
            "tabSize": 4,
            "insertSpaces": true
        },
    });
    let formatting = build_request_with_id("textDocument/formatting", params, 1);
    let _response = call_request(service, formatting.clone()).await;
    formatting
}

pub(crate) async fn highlight_request(service: &mut LspService<ServerState>, uri: &Url) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
        "position": {
            "line": 45,
            "character": 37
        }
    });
    let highlight = build_request_with_id("textDocument/documentHighlight", params, 1);
    let response = call_request(service, highlight.clone()).await;
    let expected = Response::from_ok(
        1.into(),
        json!([
            {
                "range": {
                    "end": {
                        "character": 10,
                        "line": 10
                    },
                    "start": {
                        "character": 4,
                        "line": 10
                    }
                }
            },
            {
                "range": {
                    "end": {
                        "character": 41,
                        "line": 45
                    },
                    "start": {
                        "character": 35,
                        "line": 45
                    }
                }
            },
        ]),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    highlight
}

pub(crate) async fn code_lens_request(service: &mut LspService<ServerState>, uri: &Url) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
    });
    let code_lens = build_request_with_id("textDocument/codeLens", params, 1);
    let response = call_request(service, code_lens.clone()).await;
    let actual_results = extract_result_array(response);
    let expected_results = vec![
        json!({
          "command": {
            "arguments": [
              {
                "name": "test_bar"
              }
            ],
            "command": "sway.runTests",
            "title": "▶︎ Run Test"
          },
          "range": {
            "end": {
              "character": 7,
              "line": 11
            },
            "start": {
              "character": 0,
              "line": 11
            }
          }
        }),
        json!({
          "command": {
            "arguments": [
              {
                "name": "test_foo"
              }
            ],
            "command": "sway.runTests",
            "title": "▶︎ Run Test"
          },
          "range": {
            "end": {
              "character": 7,
              "line": 6
            },
            "start": {
              "character": 0,
              "line": 6
            }
          }
        }),
        json!({
          "command": {
            "command": "sway.runScript",
            "title": "▶︎ Run"
          },
          "range": {
            "end": {
              "character": 7,
              "line": 2
            },
            "start": {
              "character": 3,
              "line": 2
            }
          }
        }),
    ];

    assert_eq!(actual_results.len(), expected_results.len());
    for expected in expected_results.iter() {
        assert!(
            actual_results.contains(expected),
            "Expected {actual_results:?} to contain {expected:?}"
        );
    }
    code_lens
}

pub(crate) async fn completion_request(
    service: &mut LspService<ServerState>,
    uri: &Url,
) -> Request {
    let params = json!({
        "textDocument": {
          "uri": uri
        },
        "position": {
          "line": 19,
          "character": 8
        },
        "context": {
          "triggerKind": 2,
          "triggerCharacter": "."
        }
    });
    let completion = build_request_with_id("textDocument/completion", params, 1);
    let response = call_request(service, completion.clone()).await;
    let actual_results = extract_result_array(response);
    let expected_results = vec![
        json!({
          "kind": 5,
          "label": "a",
          "labelDetails": {
            "description": "bool"
          }
        }),
        json!({
          "kind": 2,
          "label": "get(…)",
          "labelDetails": {
            "description": "fn(self, MyStruct) -> MyStruct"
          },
          "textEdit": {
            "newText": "get(foo)",
            "range": {
              "end": {
                "character": 8,
                "line": 19
              },
              "start": {
                "character": 8,
                "line": 19
              }
            }
          }
        }),
    ];

    assert_eq!(actual_results.len(), expected_results.len());
    for expected in expected_results.iter() {
        assert!(
            actual_results.contains(expected),
            "Expected {actual_results:?} to contain {expected:?}"
        );
    }
    completion
}

pub(crate) fn definition_check<'a>(server: &ServerState, go_to: &'a GotoDefinition<'a>) {
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
    let res = request::handle_goto_definition(&server, params.clone()).unwrap();
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

pub(crate) fn hover_request<'a>(server: &ServerState, hover_docs: &'a HoverDocumentation<'a>) {
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
    let res = request::handle_hover(&server, params.clone()).unwrap();
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

pub(crate) fn prepare_rename_request<'a>(
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
    request::handle_prepare_rename(&server, params.clone()).unwrap()
}

pub(crate) fn rename_request<'a>(
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
    let worspace_edit = request::handle_rename(&server, params.clone()).unwrap();
    worspace_edit.unwrap()
}
