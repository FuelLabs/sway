#[cfg(test)]
mod tests {
    use crate::server::{self, Backend};
    use crate::utils::test::{
        assert_server_requests, dir_contains_forc_manifest, doc_comments_dir, e2e_language_dir,
        e2e_test_dir, get_fixture, runnables_test_dir, sway_workspace_dir, test_fixtures_dir,
    };
    use assert_json_diff::assert_json_eq;
    use serde_json::json;
    use std::{
        borrow::Cow,
        fs,
        io::Read,
        path::{Path, PathBuf},
    };
    use tower::{Service, ServiceExt};
    use tower_lsp::{
        jsonrpc::{self, Id, Request, Response},
        lsp_types::*,
        ExitedError, LspService,
    };

    /// Holds the information needed to check the response of a goto definition request.
    struct GotoDefintion<'a> {
        req_uri: &'a Url,
        req_line: i32,
        req_char: i32,
        def_line: i32,
        def_start_char: i32,
        def_end_char: i32,
        def_path: &'a str,
    }

    /// Contains data required to evaluate a hover request response.
    struct HoverDocumentation<'a> {
        req_uri: &'a Url,
        req_line: i32,
        req_char: i32,
        documentation: &'a str,
    }

    fn load_sway_example(src_path: PathBuf) -> (Url, String) {
        let mut file = fs::File::open(&src_path).unwrap();
        let mut sway_program = String::new();
        file.read_to_string(&mut sway_program).unwrap();

        let uri = Url::from_file_path(src_path).unwrap();

        (uri, sway_program)
    }

    fn build_request_with_id(
        method: impl Into<Cow<'static, str>>,
        params: serde_json::Value,
        id: impl Into<Id>,
    ) -> Request {
        Request::build(method).params(params).id(id).finish()
    }

    async fn call_request(
        service: &mut LspService<Backend>,
        req: Request,
    ) -> Result<Option<Response>, ExitedError> {
        service.ready().await?.call(req).await
    }

    async fn initialize_request(service: &mut LspService<Backend>) -> Request {
        let params = json!({ "capabilities": server::capabilities() });
        let initialize = build_request_with_id("initialize", params, 1);
        let response = call_request(service, initialize.clone()).await;
        let expected =
            Response::from_ok(1.into(), json!({ "capabilities": server::capabilities() }));
        assert_json_eq!(expected, response.ok().unwrap());
        initialize
    }

    async fn initialized_notification(service: &mut LspService<Backend>) {
        let initialized = Request::build("initialized").finish();
        let response = call_request(service, initialized).await;
        assert_eq!(response, Ok(None));
    }

    async fn shutdown_request(service: &mut LspService<Backend>) -> Request {
        let shutdown = Request::build("shutdown").id(1).finish();
        let response = call_request(service, shutdown.clone()).await;
        let expected = Response::from_ok(1.into(), json!(null));
        assert_json_eq!(expected, response.ok().unwrap());
        shutdown
    }

    async fn exit_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("exit").finish();
        let response = call_request(service, exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    async fn did_open_notification(service: &mut LspService<Backend>, uri: &Url, text: &str) {
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

    async fn did_change_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

    async fn did_close_notification(service: &mut LspService<Backend>) {
        let exit = Request::build("textDocument/didClose").finish();
        let response = call_request(service, exit.clone()).await;
        assert_eq!(response, Ok(None));
    }

    async fn show_ast_request(
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

    async fn semantic_tokens_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
        });
        let semantic_tokens = build_request_with_id("textDocument/semanticTokens/full", params, 1);
        let _response = call_request(service, semantic_tokens.clone()).await;
        semantic_tokens
    }

    async fn document_symbol_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
        let params = json!({
            "textDocument": {
                "uri": uri,
            },
        });
        let document_symbol = build_request_with_id("textDocument/documentSymbol", params, 1);
        let _response = call_request(service, document_symbol.clone()).await;
        document_symbol
    }

    fn definition_request(uri: &Url, token_line: i32, token_char: i32, id: i64) -> Request {
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

    async fn definition_check<'a>(
        service: &mut LspService<Backend>,
        go_to: &'a GotoDefintion<'a>,
        id: i64,
    ) -> Request {
        let definition = definition_request(go_to.req_uri, go_to.req_line, go_to.req_char, id);
        let response = call_request(service, definition.clone())
            .await
            .unwrap()
            .unwrap();
        let value = response.result().unwrap().clone();
        if let GotoDefinitionResponse::Scalar(response) = serde_json::from_value(value).unwrap() {
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
            panic!("Expected GotoDefinitionResponse::Scalar");
        }
        definition
    }

    async fn hover_request<'a>(
        service: &mut LspService<Backend>,
        hover_docs: &'a HoverDocumentation<'a>,
        id: i64,
    ) -> Request {
        let params = json!({
            "textDocument": {
                "uri": hover_docs.req_uri,
            },
            "position": {
                "line": hover_docs.req_line,
                "character": hover_docs.req_char
            }
        });
        let hover = build_request_with_id("textDocument/hover", params, id);
        let response = call_request(service, hover.clone()).await.unwrap().unwrap();
        let value = response.result().unwrap().clone();
        let hover_res: Hover = serde_json::from_value(value).unwrap();

        if let HoverContents::Markup(markup_content) = hover_res.contents {
            assert_eq!(hover_docs.documentation, markup_content.value);
        } else {
            panic!("Expected HoverContents::Markup");
        }
        hover
    }

    async fn format_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

    async fn highlight_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

    async fn code_action_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

    async fn code_lens_request(service: &mut LspService<Backend>, uri: &Url) -> Request {
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

    async fn init_and_open(service: &mut LspService<Backend>, entry_point: PathBuf) -> Url {
        let _ = initialize_request(service).await;
        initialized_notification(service).await;
        let (uri, sway_program) = load_sway_example(entry_point);
        did_open_notification(service, &uri, &sway_program).await;
        uri
    }

    async fn shutdown_and_exit(service: &mut LspService<Backend>) {
        let _ = shutdown_request(service).await;
        exit_notification(service).await;
    }

    // This method iterates over all of the examples in the e2e langauge should_pass dir
    // and saves the lexed, parsed, and typed ASTs to the users home directory.
    // This makes it easy to grep for certain compiler types to inspect their use cases,
    // providing necessary context when working on the traversal modules.
    #[allow(unused)]
    //#[tokio::test]
    async fn write_all_example_asts() {
        let (mut service, _) = LspService::build(Backend::new)
            .custom_method("sway/show_ast", Backend::show_ast)
            .finish();
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;

        let ast_folder = dirs::home_dir()
            .expect("could not get users home directory")
            .join("sway_asts");
        let _ = fs::create_dir(&ast_folder);
        let e2e_dir = sway_workspace_dir().join(e2e_language_dir());
        let mut entries = fs::read_dir(&e2e_dir)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();

        // The order in which `read_dir` returns entries is not guaranteed. If reproducible
        // ordering is required the entries should be explicitly sorted.
        entries.sort();

        for entry in entries {
            let manifest_dir = entry;
            let example_name = manifest_dir.file_name().unwrap();
            if manifest_dir.is_dir() {
                let example_dir = ast_folder.join(example_name);
                if !dir_contains_forc_manifest(manifest_dir.as_path()) {
                    continue;
                }
                match fs::create_dir(&example_dir) {
                    Ok(_) => (),
                    Err(_) => continue,
                }

                let example_dir = Some(Url::from_file_path(example_dir).unwrap());
                let (uri, sway_program) = load_sway_example(manifest_dir.join("src/main.sw"));
                did_open_notification(&mut service, &uri, &sway_program).await;
                let _ = show_ast_request(&mut service, &uri, "lexed", example_dir.clone()).await;
                let _ = show_ast_request(&mut service, &uri, "parsed", example_dir.clone()).await;
                let _ = show_ast_request(&mut service, &uri, "typed", example_dir).await;
            }
        }
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn initialize() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
    }

    #[tokio::test]
    async fn initialized() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
    }

    #[tokio::test]
    async fn initializes_only_once() {
        let (mut service, _) = LspService::new(Backend::new);
        let initialize = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
        let response = call_request(&mut service, initialize).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    async fn shutdown() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        initialized_notification(&mut service).await;
        let shutdown = shutdown_request(&mut service).await;
        let response = call_request(&mut service, shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
        exit_notification(&mut service).await;
    }

    #[tokio::test]
    async fn refuses_requests_after_shutdown() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = initialize_request(&mut service).await;
        let shutdown = shutdown_request(&mut service).await;
        let response = call_request(&mut service, shutdown).await;
        let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
        assert_eq!(response, Ok(Some(err)));
    }

    #[tokio::test]
    async fn did_open() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = init_and_open(&mut service, e2e_test_dir().join("src/main.sw")).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn did_close() {
        let (mut service, _) = LspService::new(Backend::new);
        let _ = init_and_open(&mut service, e2e_test_dir().join("src/main.sw")).await;
        did_close_notification(&mut service).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn did_change() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
        let _ = did_change_request(&mut service, &uri).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn lsp_syncs_with_workspace_edits() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 44,
            req_char: 24,
            def_line: 19,
            def_start_char: 7,
            def_end_char: 11,
            def_path: uri.as_str(),
        };
        let _ = definition_check(&mut service, &go_to, 1).await;
        let _ = did_change_request(&mut service, &uri).await;
        go_to.def_line = 20;
        definition_check_with_req_offset(&mut service, &mut go_to, 45, 24, 2).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn show_ast() {
        let (mut service, _) = LspService::build(Backend::new)
            .custom_method("sway/show_ast", Backend::show_ast)
            .finish();

        let uri = init_and_open(&mut service, e2e_test_dir().join("src/main.sw")).await;
        let _ = show_ast_request(&mut service, &uri, "typed", None).await;
        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 44,
            req_char: 24,
            def_line: 19,
            def_start_char: 7,
            def_end_char: 11,
            def_path: uri.as_str(),
        };
        let _ = definition_check(&mut service, &go_to, 1).await;
        shutdown_and_exit(&mut service).await;
    }

    async fn definition_check_with_req_offset<'a>(
        service: &mut LspService<Backend>,
        go_to: &mut GotoDefintion<'a>,
        req_line: i32,
        req_char: i32,
        id: i64,
    ) {
        go_to.req_line = req_line;
        go_to.req_char = req_char;
        let _ = definition_check(service, go_to, id).await;
    }

    #[tokio::test]
    async fn go_to_definition_inside_turbofish() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/turbofish/src/main.sw"),
        )
        .await;

        let mut opt_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 15,
            req_char: 12,
            def_line: 80,
            def_start_char: 9,
            def_end_char: 15,
            def_path: "sway-lib-std/src/option.sw",
        };
        // option.sw
        let _ = definition_check(&mut service, &opt_go_to, 1).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 16, 17, 2).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 17, 29, 3).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 18, 19, 4).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 20, 13, 5).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 21, 19, 6).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 22, 29, 7).await;
        definition_check_with_req_offset(&mut service, &mut opt_go_to, 23, 18, 8).await;

        let mut res_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 20,
            req_char: 19,
            def_line: 60,
            def_start_char: 9,
            def_end_char: 15,
            def_path: "sway-lib-std/src/result.sw",
        };
        // result.sw
        let _ = definition_check(&mut service, &res_go_to, 9).await;
        definition_check_with_req_offset(&mut service, &mut res_go_to, 21, 25, 10).await;
        definition_check_with_req_offset(&mut service, &mut res_go_to, 22, 36, 11).await;
        definition_check_with_req_offset(&mut service, &mut res_go_to, 23, 27, 12).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_modules() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/modules/src/lib.sw"),
        )
        .await;

        let opt_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 2,
            req_char: 6,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 16,
            def_path: "sway-lsp/test/fixtures/tokens/modules/src/test_mod.sw",
        };
        // dep test_mod;
        let _ = definition_check(&mut service, &opt_go_to, 2).await;

        let opt_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 3,
            req_char: 6,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/modules/src/dir_mod/mod.sw",
        };
        // dep dir_mod/mod;
        let _ = definition_check(&mut service, &opt_go_to, 3).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_paths() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/paths/src/main.sw"),
        )
        .await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 13,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 11,
            def_path: "sway-lib-std/src/lib.sw",
        };
        // std
        let _ = definition_check(&mut service, &go_to, 1).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 12, 14, 2).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 18, 5, 3).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 24, 13, 4).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 7, 5, 5).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 19,
            def_line: 74,
            def_start_char: 8,
            def_end_char: 14,
            def_path: "sway-lib-std/src/option.sw",
        };
        // option
        let _ = definition_check(&mut service, &go_to, 6).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 10,
            req_char: 27,
            def_line: 80,
            def_start_char: 9,
            def_end_char: 15,
            def_path: "sway-lib-std/src/option.sw",
        };
        // Option
        let _ = definition_check(&mut service, &go_to, 7).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 11, 14, 8).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 12,
            req_char: 17,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 10,
            def_path: "sway-lib-std/src/vm/mod.sw",
        };
        // vm
        let _ = definition_check(&mut service, &go_to, 9).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 12,
            req_char: 22,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 11,
            def_path: "sway-lib-std/src/vm/evm/mod.sw",
        };
        // evm
        let _ = definition_check(&mut service, &go_to, 10).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 12,
            req_char: 27,
            def_line: 1,
            def_start_char: 8,
            def_end_char: 19,
            def_path: "sway-lib-std/src/vm/evm/evm_address.sw",
        };
        // evm_address
        let _ = definition_check(&mut service, &go_to, 11).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 12,
            req_char: 42,
            def_line: 7,
            def_start_char: 11,
            def_end_char: 21,
            def_path: "sway-lib-std/src/vm/evm/evm_address.sw",
        };
        // EvmAddress
        let _ = definition_check(&mut service, &go_to, 12).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 16,
            req_char: 6,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 16,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // test_mod
        let _ = definition_check(&mut service, &go_to, 13).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 22, 7, 14).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 5, 5, 15).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 16,
            req_char: 16,
            def_line: 2,
            def_start_char: 7,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // test_fun
        let _ = definition_check(&mut service, &go_to, 16).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 17,
            req_char: 8,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 16,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/deep_mod.sw",
        };
        // deep_mod
        let _ = definition_check(&mut service, &go_to, 17).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 6, 6, 18).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 17,
            req_char: 18,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 18,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
        };
        // deeper_mod
        let _ = definition_check(&mut service, &go_to, 19).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 6, 16, 20).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 17,
            req_char: 29,
            def_line: 2,
            def_start_char: 7,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
        };
        // deep_fun
        let _ = definition_check(&mut service, &go_to, 21).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 6, 28, 22).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 18,
            req_char: 11,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 14,
            def_path: "sway-lib-std/src/assert.sw",
        };
        // assert
        let _ = definition_check(&mut service, &go_to, 23).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 19,
            req_char: 13,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 12,
            def_path: "sway-lib-core/src/lib.sw",
        };
        // core
        let _ = definition_check(&mut service, &go_to, 24).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 19,
            req_char: 21,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 18,
            def_path: "sway-lib-core/src/primitives.sw",
        };
        // primitives
        let _ = definition_check(&mut service, &go_to, 25).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 25, 20, 26).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 5,
            req_char: 14,
            def_line: 4,
            def_start_char: 11,
            def_end_char: 12,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // A def
        let _ = definition_check(&mut service, &go_to, 27).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 21,
            req_char: 4,
            def_line: 6,
            def_start_char: 5,
            def_end_char: 6,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // A impl
        let _ = definition_check(&mut service, &go_to, 28).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 22, 14, 29).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 21,
            req_char: 7,
            def_line: 7,
            def_start_char: 11,
            def_end_char: 14,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/test_mod.sw",
        };
        // fun
        let _ = definition_check(&mut service, &go_to, 30).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 22, 18, 31).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 24,
            req_char: 20,
            def_line: 0,
            def_start_char: 8,
            def_end_char: 17,
            def_path: "sway-lib-std/src/constants.sw",
        };
        // constants
        let _ = definition_check(&mut service, &go_to, 32).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 7, 11, 33).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 7, 23, 34).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 24,
            req_char: 31,
            def_line: 5,
            def_start_char: 10,
            def_end_char: 19,
            def_path: "sway-lib-std/src/constants.sw",
        };
        // ZERO_B256
        let _ = definition_check(&mut service, &go_to, 35).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 7, 31, 36).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 19,
            req_char: 31,
            def_line: 2,
            def_start_char: 5,
            def_end_char: 8,
            def_path: "sway-lib-core/src/primitives.sw",
        };
        // u64
        let _ = definition_check(&mut service, &go_to, 37).await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 13,
            req_char: 17,
            def_line: 74,
            def_start_char: 5,
            def_end_char: 9,
            def_path: "sway-lib-core/src/primitives.sw",
        };
        // b256
        let _ = definition_check(&mut service, &go_to, 38).await;
        definition_check_with_req_offset(&mut service, &mut go_to, 25, 31, 39).await;

        let go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 6,
            req_char: 39,
            def_line: 6,
            def_start_char: 38,
            def_end_char: 42,
            def_path: "sway-lsp/test/fixtures/tokens/paths/src/main.sw",
        };
        // dfun
        let _ = definition_check(&mut service, &go_to, 40).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_traits() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/traits/src/main.sw"),
        )
        .await;

        let mut trait_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 6,
            req_char: 10,
            def_line: 2,
            def_start_char: 10,
            def_end_char: 15,
            def_path: "sway-lsp/test/fixtures/tokens/traits/src/traits.sw",
        };

        let _ = definition_check(&mut service, &trait_go_to, 1).await;
        definition_check_with_req_offset(&mut service, &mut trait_go_to, 7, 10, 2).await;
        definition_check_with_req_offset(&mut service, &mut trait_go_to, 10, 6, 3).await;
        trait_go_to.req_line = 7;
        trait_go_to.req_char = 20;
        trait_go_to.def_line = 3;
        let _ = definition_check(&mut service, &trait_go_to, 3).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_variables() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/variables/src/main.sw"),
        )
        .await;

        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 23,
            req_char: 26,
            def_line: 22,
            def_start_char: 8,
            def_end_char: 17,
            def_path: uri.as_str(),
        };
        // Variable expressions
        let _ = definition_check(&mut service, &go_to, 1).await;

        // Function arguments
        go_to.def_line = 23;
        definition_check_with_req_offset(&mut service, &mut go_to, 28, 35, 2).await;

        // Struct fields
        go_to.def_line = 22;
        definition_check_with_req_offset(&mut service, &mut go_to, 31, 45, 3).await;

        // Enum fields
        go_to.def_line = 22;
        definition_check_with_req_offset(&mut service, &mut go_to, 34, 39, 4).await;

        // Tuple elements
        go_to.def_line = 24;
        definition_check_with_req_offset(&mut service, &mut go_to, 37, 20, 5).await;

        // Array elements
        go_to.def_line = 25;
        definition_check_with_req_offset(&mut service, &mut go_to, 40, 20, 6).await;

        // Scoped declarations
        go_to.def_line = 44;
        go_to.def_start_char = 12;
        go_to.def_end_char = 21;
        definition_check_with_req_offset(&mut service, &mut go_to, 45, 13, 7).await;

        // If let scopes
        go_to.def_line = 50;
        go_to.def_start_char = 38;
        go_to.def_end_char = 39;
        definition_check_with_req_offset(&mut service, &mut go_to, 50, 47, 8).await;

        // Shadowing
        go_to.def_line = 50;
        go_to.def_start_char = 8;
        go_to.def_end_char = 17;
        definition_check_with_req_offset(&mut service, &mut go_to, 53, 29, 9).await;

        // Variable type ascriptions
        go_to.def_line = 6;
        go_to.def_start_char = 5;
        go_to.def_end_char = 16;
        definition_check_with_req_offset(&mut service, &mut go_to, 56, 21, 10).await;

        shutdown_and_exit(&mut service).await;
    }

    #[tokio::test]
    async fn go_to_definition_for_consts() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/consts/src/main.sw"),
        )
        .await;

        // value: TyExpression
        let mut contract_go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 9,
            req_char: 24,
            def_line: 18,
            def_start_char: 5,
            def_end_char: 9,
            def_path: "sway-lib-std/src/contract_id.sw",
        };
        let _ = definition_check(&mut service, &contract_go_to, 1).await;

        contract_go_to.req_char = 34;
        contract_go_to.def_line = 19;
        contract_go_to.def_start_char = 7;
        contract_go_to.def_end_char = 11;
        let _ = definition_check(&mut service, &contract_go_to, 2).await;

        // Constants defined in the same module
        let mut go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 19,
            req_char: 34,
            def_line: 6,
            def_start_char: 6,
            def_end_char: 16,
            def_path: uri.as_str(),
        };
        let _ = definition_check(&mut service, &contract_go_to, 3).await;

        go_to.def_line = 9;
        definition_check_with_req_offset(&mut service, &mut go_to, 20, 29, 4).await;

        // Constants defined in a different module
        go_to = GotoDefintion {
            req_uri: &uri,
            req_line: 23,
            req_char: 73,
            def_line: 12,
            def_start_char: 10,
            def_end_char: 20,
            def_path: "consts/src/more_consts.sw",
        };
        let _ = definition_check(&mut service, &go_to, 5).await;

        go_to.def_line = 13;
        go_to.def_start_char = 10;
        go_to.def_end_char = 18;
        definition_check_with_req_offset(&mut service, &mut go_to, 24, 31, 6).await;

        // Constants with type ascriptions
        go_to.def_line = 6;
        go_to.def_start_char = 5;
        go_to.def_end_char = 9;
        definition_check_with_req_offset(&mut service, &mut go_to, 10, 17, 7).await;
    }

    #[tokio::test]
    async fn hover_docs_for_consts() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(
            &mut service,
            test_fixtures_dir().join("tokens/consts/src/main.sw"),
        )
        .await;

        let mut hover = HoverDocumentation {
            req_uri: &uri,
            req_line: 19,
            req_char: 33,
            documentation: " documentation for CONSTANT_1",
        };

        let _ = hover_request(&mut service, &hover, 1).await;
        hover.req_char = 49;
        hover.documentation = " CONSTANT_2 has a value of 200";
        let _ = hover_request(&mut service, &hover, 2).await;
    }

    #[tokio::test]
    async fn hover_docs_with_code_examples() {
        let (mut service, _) = LspService::new(Backend::new);
        let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;

        let hover = HoverDocumentation {
            req_uri: &uri,
            req_line: 44,
            req_char: 24,
            documentation: "```sway\nstruct Data\n```\n---\n Struct holding:\n\n 1. A `value` of type `NumberOrString`\n 2. An `address` of type `u64`",
        };
        let _ = hover_request(&mut service, &hover, 1).await;
    }

    #[tokio::test]
    async fn publish_diagnostics_dead_code_warning() {
        let (mut service, socket) = LspService::new(Backend::new);
        let fixture = get_fixture(test_fixtures_dir().join("diagnostics/dead_code/expected.json"));
        let expected_requests = vec![fixture];
        let socket_handle = assert_server_requests(socket, expected_requests, None).await;
        let _ = init_and_open(
            &mut service,
            test_fixtures_dir().join("diagnostics/dead_code/src/main.sw"),
        )
        .await;
        socket_handle
            .await
            .unwrap_or_else(|e| panic!("Test failed: {e:?}"));
        shutdown_and_exit(&mut service).await;
    }

    // This macro allows us to spin up a server / client for testing
    // It initializes and performs the necessary handshake and then loads
    // the sway example that was passed into `example_dir`.
    // It then runs the specific capability to test before gracefully shutting down.
    // The capability argument is an async function.
    macro_rules! test_lsp_capability {
        ($entry_point:expr, $capability:expr) => {{
            let (mut service, _) = LspService::new(Backend::new);
            let uri = init_and_open(&mut service, $entry_point).await;
            // Call the specific LSP capability function that was passed in.
            let _ = $capability(&mut service, &uri).await;
            shutdown_and_exit(&mut service).await;
        }};
    }

    macro_rules! lsp_capability_test {
        ($test:ident, $capability:expr, $entry_path:expr) => {
            #[tokio::test]
            async fn $test() {
                test_lsp_capability!($entry_path, $capability);
            }
        };
    }

    lsp_capability_test!(
        semantic_tokens,
        semantic_tokens_request,
        doc_comments_dir().join("src/main.sw")
    );
    lsp_capability_test!(
        document_symbol,
        document_symbol_request,
        doc_comments_dir().join("src/main.sw")
    );
    lsp_capability_test!(
        format,
        format_request,
        doc_comments_dir().join("src/main.sw")
    );
    lsp_capability_test!(
        highlight,
        highlight_request,
        doc_comments_dir().join("src/main.sw")
    );
    lsp_capability_test!(
        code_action,
        code_action_request,
        doc_comments_dir().join("src/main.sw")
    );
    lsp_capability_test!(
        code_lens,
        code_lens_request,
        runnables_test_dir().join("src/main.sw")
    );
}
