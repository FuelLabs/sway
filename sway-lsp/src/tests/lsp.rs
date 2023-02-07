use crate::server::{self, Backend};
use crate::utils::test::{
    assert_server_requests, dir_contains_forc_manifest, doc_comments_dir, e2e_language_dir,
    e2e_test_dir, get_fixture, runnables_test_dir, sway_workspace_dir, test_fixtures_dir,
};
use assert_json_diff::assert_json_eq;
use serde_json::json;
use std::{borrow::Cow, path::Path};
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
    service: &mut LspService<Backend>,
    req: Request,
) -> Result<Option<Response>, ExitedError> {
    service.ready().await?.call(req).await
}

pub(crate) async fn initialize_request(service: &mut LspService<Backend>) -> Request {
    let params = json!({ "capabilities": server::capabilities() });
    let initialize = build_request_with_id("initialize", params, 1);
    let response = call_request(service, initialize.clone()).await;
    let expected = Response::from_ok(1.into(), json!({ "capabilities": server::capabilities() }));
    assert_json_eq!(expected, response.ok().unwrap());
    initialize
}

pub(crate) async fn initialized_notification(service: &mut LspService<Backend>) {
    let initialized = Request::build("initialized").finish();
    let response = call_request(service, initialized).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn shutdown_request(service: &mut LspService<Backend>) -> Request {
    let shutdown = Request::build("shutdown").id(1).finish();
    let response = call_request(service, shutdown.clone()).await;
    let expected = Response::from_ok(1.into(), json!(null));
    assert_json_eq!(expected, response.ok().unwrap());
    shutdown
}

pub(crate) async fn exit_notification(service: &mut LspService<Backend>) {
    let exit = Request::build("exit").finish();
    let response = call_request(service, exit.clone()).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn did_open_notification(
    service: &mut LspService<Backend>,
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

pub(crate) async fn did_change_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

pub(crate) async fn did_close_notification(service: &mut LspService<Backend>) {
    let exit = Request::build("textDocument/didClose").finish();
    let response = call_request(service, exit.clone()).await;
    assert_eq!(response, Ok(None));
}

pub(crate) async fn show_ast_request(
    service: &mut LspService<Backend>,
    uri: &Url,
    ast_kind: &str,
    save_path: Option<Url>,
) -> Request {
    // The path where the AST will be written to.
    // If no path is provided, the default path is "/tmp"
    let save_path = match save_path {
        Some(path) => path,
        None => Url::from_file_path(Path::new("/tmp")).unwrap(),
    };
    let params = json!({
        "textDocument": {
            "uri": uri
        },
        "astKind": ast_kind,
        "savePath": save_path,
    });
    let show_ast = build_request_with_id("sway/show_ast", params, 1);
    let response = call_request(service, show_ast.clone()).await;
    let expected = Response::from_ok(
        1.into(),
        json!({ "uri": format!("{save_path}/{ast_kind}.rs") }),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    show_ast
}

pub(crate) async fn semantic_tokens_request(
    service: &mut LspService<Backend>,
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
    service: &mut LspService<Backend>,
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

pub(crate) fn definition_request(uri: &Url, token_line: i32, token_char: i32, id: i64) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
        "position": {
            "line": token_line,
            "character": token_char,
        }
    });
    build_request_with_id("textDocument/definition", params, id)
}

pub(crate) async fn format_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

pub(crate) async fn highlight_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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
        json!([{
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
            }
        ]),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    highlight
}

pub(crate) async fn code_action_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
        "range" : {
            "start":{
                "line": 27,
                "character": 4
            },
            "end":{
                "line": 27,
                "character": 9
            },
        },
        "context": {
            "diagnostics": [],
            "triggerKind": 2
        }
    });
    let code_action = build_request_with_id("textDocument/codeAction", params, 1);
    let response = call_request(service, code_action.clone()).await;
    let uri_string = uri.to_string();
    let expected = Response::from_ok(
        1.into(),
        json!([{
            "data": uri,
            "edit": {
              "changes": {
                uri_string: [
                  {
                    "newText": "\nimpl FooABI for Contract {\n    /// This is the `main` method on the `FooABI` abi\n    fn main() -> u64 {}\n}\n",
                    "range": {
                      "end": {
                        "character": 0,
                        "line": 31
                      },
                      "start": {
                        "character": 0,
                        "line": 31
                      }
                    }
                  }
                ]
              }
            },
            "kind": "refactor",
            "title": "Generate impl for contract"
        }]),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    code_action
}

pub(crate) async fn code_lens_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
    });
    let code_lens = build_request_with_id("textDocument/codeLens", params, 1);
    let response = call_request(service, code_lens.clone()).await;
    let actual_results = response
        .unwrap()
        .unwrap()
        .into_parts()
        .1
        .ok()
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
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
