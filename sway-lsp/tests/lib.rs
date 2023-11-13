pub mod integration;

use crate::integration::{code_actions, lsp};
use lsp_types::*;
use std::{fs, path::PathBuf};
use sway_lsp::{
    handlers::{notification, request},
    server_state::ServerState,
};
use sway_lsp_test_utils::{
    assert_server_requests, dir_contains_forc_manifest, doc_comments_dir, e2e_language_dir,
    e2e_test_dir, generic_impl_self_dir, get_fixture, load_sway_example, runnables_test_dir,
    self_impl_reassignment_dir, sway_workspace_dir, test_fixtures_dir,
};
use tower_lsp::LspService;

/// Holds the information needed to check the response of a goto definition request.
#[derive(Debug)]
pub(crate) struct GotoDefinition<'a> {
    req_uri: &'a Url,
    req_line: u32,
    req_char: u32,
    def_line: u32,
    def_start_char: u32,
    def_end_char: u32,
    def_path: &'a str,
}

/// Contains data required to evaluate a hover request response.
pub(crate) struct HoverDocumentation<'a> {
    req_uri: &'a Url,
    req_line: u32,
    req_char: u32,
    documentation: Vec<&'a str>,
}

/// Contains data required to evaluate a rename request.
pub(crate) struct Rename<'a> {
    req_uri: &'a Url,
    req_line: u32,
    req_char: u32,
    new_name: &'a str,
}

async fn open(server: &ServerState, entry_point: PathBuf) -> Url {
    let (uri, sway_program) = load_sway_example(entry_point);
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "sway".to_string(),
            version: 1,
            text: sway_program,
        },
    };
    let res = notification::handle_did_open_text_document(server, params).await;
    assert!(res.is_ok());
    uri
}

async fn init_and_open(service: &mut LspService<ServerState>, entry_point: PathBuf) -> Url {
    let _ = lsp::initialize_request(service).await;
    lsp::initialized_notification(service).await;
    let (uri, sway_program) = load_sway_example(entry_point);
    lsp::did_open_notification(service, &uri, &sway_program).await;
    uri
}

async fn shutdown_and_exit(service: &mut LspService<ServerState>) {
    let _ = lsp::shutdown_request(service).await;
    lsp::exit_notification(service).await;
}

#[tokio::test]
async fn initialize() {
    let server = ServerState::default();
    let params = InitializeParams {
        initialization_options: None,
        ..Default::default()
    };
    let _ = request::handle_initialize(&server, params);
}

#[tokio::test]
async fn did_open() {
    let server = ServerState::default();
    let _ = open(&server, e2e_test_dir().join("src/main.sw")).await;
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn did_change() {
    let (mut service, _) = LspService::new(ServerState::new);
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let _ = lsp::did_change_request(&mut service, &uri).await;
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn did_cache_test() {
    let (mut service, _) = LspService::build(ServerState::new)
        .custom_method("sway/metrics", ServerState::metrics)
        .finish();
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let _ = lsp::did_change_request(&mut service, &uri).await;
    let metrics = lsp::metrics_request(&mut service, &uri).await;
    assert!(metrics.len() >= 2);
    for (path, metrics) in metrics {
        if path.contains("sway-lib-core") || path.contains("sway-lib-std") {
            assert!(metrics.reused_modules >= 1);
        }
    }
    shutdown_and_exit(&mut service).await;
}

// #[tokio::test]
#[allow(dead_code)]
async fn did_change_stress_test() {
    let (mut service, _) = LspService::build(ServerState::new)
        .custom_method("sway/metrics", ServerState::metrics)
        .finish();
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let times = 20;
    for _ in 0..times {
        let _ = lsp::did_change_request(&mut service, &uri).await;
        let metrics = lsp::metrics_request(&mut service, &uri).await;
        for (path, metrics) in metrics {
            if path.contains("sway-lib-core") || path.contains("sway-lib-std") {
                assert!(metrics.reused_modules >= 1);
            }
        }
    }
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn lsp_syncs_with_workspace_edits() {
    let (mut service, _) = LspService::new(ServerState::new);
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 44,
        req_char: 24,
        def_line: 19,
        def_start_char: 7,
        def_end_char: 11,
        def_path: uri.as_str(),
    };
    lsp::definition_check(service.inner(), &go_to);
    let _ = lsp::did_change_request(&mut service, &uri).await;
    go_to.def_line = 20;
    lsp::definition_check_with_req_offset(service.inner(), &mut go_to, 45, 24);
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn show_ast() {
    let server = ServerState::default();
    let uri = open(&server, e2e_test_dir().join("src/main.sw")).await;
    lsp::show_ast_request(&server, &uri, "typed", None).await;
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn visualize() {
    let server = ServerState::default();
    let uri = open(&server, e2e_test_dir().join("src/main.sw")).await;
    lsp::visualize_request(&server, &uri, "build_plan").await;
    let _ = server.shutdown_server();
}

//------------------- GO TO DEFINITION -------------------//

#[tokio::test]
async fn go_to_definition() {
    let server = ServerState::default();
    let uri = open(&server, doc_comments_dir().join("src/main.sw")).await;
    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 44,
        req_char: 24,
        def_line: 19,
        def_start_char: 7,
        def_end_char: 11,
        def_path: uri.as_str(),
    };
    lsp::definition_check(&server, &go_to);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_fields() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/fields/src/main.sw"),
    )
    .await;
    let mut opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 5,
        req_char: 8,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "fixtures-std/src/option.sw",
    };
    // Option
    lsp::definition_check(&server, &opt_go_to);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 5, 16);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 9, 9);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 9, 16);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 13, 12);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 13, 19);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 13, 34);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 13, 47);

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 10,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "sway-lsp/tests/fixtures/tokens/fields/src/foo.sw",
    };
    // foo
    lsp::definition_check(&server, &opt_go_to);

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 15,
        def_line: 2,
        def_start_char: 11,
        def_end_char: 14,
        def_path: "sway-lsp/tests/fixtures/tokens/fields/src/foo.sw",
    };
    // Foo
    lsp::definition_check(&server, &opt_go_to);

    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_inside_turbofish() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/turbofish/src/main.sw"),
    )
    .await;

    let mut opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 15,
        req_char: 12,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "fixtures-std/src/option.sw",
    };
    // option.sw
    lsp::definition_check(&server, &opt_go_to);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 16, 17);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 17, 29);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 18, 19);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 20, 13);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 21, 19);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 22, 29);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 23, 18);
    lsp::definition_check_with_req_offset(&server, &mut opt_go_to, 24, 26);

    let mut res_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 20,
        req_char: 19,
        def_line: 61,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "fixtures-std/src/result.sw",
    };
    // result.sw
    lsp::definition_check(&server, &res_go_to);
    lsp::definition_check_with_req_offset(&server, &mut res_go_to, 21, 25);
    lsp::definition_check_with_req_offset(&server, &mut res_go_to, 22, 36);
    lsp::definition_check_with_req_offset(&server, &mut res_go_to, 23, 27);
    lsp::definition_check_with_req_offset(&server, &mut res_go_to, 24, 33);

    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_matches() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/matches/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 14,
        req_char: 10,
        def_line: 10,
        def_start_char: 6,
        def_end_char: 19,
        def_path: "sway-lsp/tests/fixtures/tokens/matches/src/main.sw",
    };
    // EXAMPLE_CONST
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 19, 18);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 22, 18);
    // TODO: Enable the below check once this issue is fixed: https://github.com/FuelLabs/sway/issues/5221
    // lsp::definition_check_with_req_offset(&server, &mut go_to, 22, 30);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 23, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 38);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 15,
        req_char: 13,
        def_line: 15,
        def_start_char: 8,
        def_end_char: 9,
        def_path: "sway-lsp/tests/fixtures/tokens/matches/src/main.sw",
    };
    // a => a + 1
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 25,
        req_char: 19,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "fixtures-std/src/option.sw",
    };
    // Option
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 33);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 26, 11);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 27, 11);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 27, 22);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 11);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 22);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 25,
        req_char: 27,
        def_line: 85,
        def_start_char: 4,
        def_end_char: 8,
        def_path: "fixtures-std/src/option.sw",
    };
    // Some
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 27, 17);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 17);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 30);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 26,
        req_char: 17,
        def_line: 83,
        def_start_char: 4,
        def_end_char: 8,
        def_path: "fixtures-std/src/option.sw",
    };
    // None
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 27, 30);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 34,
        req_char: 11,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 20,
        def_path: "sway-lsp/tests/fixtures/tokens/matches/src/main.sw",
    };
    // ExampleStruct
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 34,
        req_char: 26,
        def_line: 3,
        def_start_char: 4,
        def_end_char: 12,
        def_path: "sway-lsp/tests/fixtures/tokens/matches/src/main.sw",
    };
    // ExampleStruct.variable
    lsp::definition_check(&server, &go_to);

    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_modules() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/modules/src/lib.sw"),
    )
    .await;

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 2,
        req_char: 6,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "sway-lsp/tests/fixtures/tokens/modules/src/test_mod.sw",
    };
    // mod test_mod;
    lsp::definition_check(&server, &opt_go_to);

    let _ = server.shutdown_server();

    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/modules/src/test_mod.sw"),
    )
    .await;

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 2,
        req_char: 6,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "sway-lsp/tests/fixtures/tokens/modules/src/test_mod/deep_mod.sw",
    };
    // mod deep_mod;
    lsp::definition_check(&server, &opt_go_to);

    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_paths() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/paths/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 13,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/lib.sw",
    };
    // std
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 12, 14);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 18, 5);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 24, 13);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 7, 5);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 19,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/option.sw",
    };
    // option
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 27,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "fixtures-std/src/option.sw",
    };
    // Option
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 14);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 17,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/vm.sw",
    };
    // vm
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 22,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/vm/evm.sw",
    };
    // evm
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 27,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/vm/evm/evm_address.sw",
    };
    // evm_address
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 42,
        def_line: 7,
        def_start_char: 11,
        def_end_char: 21,
        def_path: "fixtures-std/src/vm/evm/evm_address.sw",
    };
    // EvmAddress
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 16,
        req_char: 6,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // test_mod
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 22, 7);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 5, 5);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 16,
        req_char: 16,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 15,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // test_fun
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 8,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod.sw",
    };
    // deep_mod
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 6, 6);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 27, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 29, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 30, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 32, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 16);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 18,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // deeper_mod
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 6, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 27, 28);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 28);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 29, 28);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 30, 28);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 32, 28);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 28);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 27,
        req_char: 38,
        def_line: 4,
        def_start_char: 9,
        def_end_char: 17,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // DeepEnum
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 38);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 29, 38);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 30, 38);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 32,
        req_char: 37,
        def_line: 9,
        def_start_char: 11,
        def_end_char: 21,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // DeepStruct
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 37);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 27,
        req_char: 48,
        def_line: 5,
        def_start_char: 4,
        def_end_char: 11,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // DeepEnum::Variant
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 48);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 29,
        req_char: 48,
        def_line: 6,
        def_start_char: 4,
        def_end_char: 10,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // DeepEnum::Number
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 30, 48);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 29,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 15,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // deep_fun
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 6, 28);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 18,
        req_char: 11,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/assert.sw",
    };
    // assert
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 19,
        req_char: 13,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-core/src/lib.sw",
    };
    // core
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 19,
        req_char: 21,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-core/src/primitives.sw",
    };
    // primitives
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 20);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 5,
        req_char: 14,
        def_line: 4,
        def_start_char: 11,
        def_end_char: 12,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // A def
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 21,
        req_char: 4,
        def_line: 4,
        def_start_char: 11,
        def_end_char: 12,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // A impl
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 22, 14);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 21,
        req_char: 7,
        def_line: 7,
        def_start_char: 11,
        def_end_char: 14,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // fun
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 22, 18);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 20,
        def_line: 0,
        def_start_char: 0,
        def_end_char: 0,
        def_path: "fixtures-std/src/constants.sw",
    };
    // constants
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 7, 11);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 7, 23);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 31,
        def_line: 33,
        def_start_char: 10,
        def_end_char: 19,
        def_path: "fixtures-std/src/constants.sw",
    };
    // ZERO_B256
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 7, 31);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 19,
        req_char: 37,
        def_line: 17,
        def_start_char: 11,
        def_end_char: 14,
        def_path: "fixtures-core/src/primitives.sw",
    };
    // u64::min()
    lsp::definition_check(&server, &go_to);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 13,
        req_char: 22,
        def_line: 247,
        def_start_char: 11,
        def_end_char: 14,
        def_path: "fixtures-core/src/primitives.sw",
    };
    // b256::min()
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 38);

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/4211 is fixed.
    // let go_to = GotoDefinition {
    //     req_uri: &uri,
    //     req_line: 6,
    //     req_char: 39,
    //     def_line: 2,
    //     def_start_char: 7,
    //     def_end_char: 15,
    //     def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    // };
    // dfun
    // lsp::definition_check(&server, &go_to);

    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_traits() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/traits/src/main.sw"),
    )
    .await;

    let mut trait_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 10,
        def_line: 2,
        def_start_char: 10,
        def_end_char: 15,
        def_path: "sway-lsp/tests/fixtures/tokens/traits/src/traits.sw",
    };

    lsp::definition_check(&server, &trait_go_to);
    lsp::definition_check_with_req_offset(&server, &mut trait_go_to, 7, 10);
    lsp::definition_check_with_req_offset(&server, &mut trait_go_to, 10, 6);
    trait_go_to.req_line = 7;
    trait_go_to.req_char = 20;
    trait_go_to.def_line = 3;
    lsp::definition_check(&server, &trait_go_to);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_variables() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/variables/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 20,
        req_char: 34,
        def_line: 19,
        def_start_char: 8,
        def_end_char: 17,
        def_path: uri.as_str(),
    };
    // Variable expressions
    lsp::definition_check(&server, &go_to);

    // Function arguments
    go_to.def_line = 20;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 35);

    // Struct fields
    go_to.def_line = 19;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 28, 45);

    // Enum fields
    go_to.def_line = 19;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 31, 39);

    // Tuple elements
    go_to.def_line = 21;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 34, 20);

    // Array elements
    go_to.def_line = 22;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 37, 20);

    // Scoped declarations
    go_to.def_line = 41;
    go_to.def_start_char = 12;
    go_to.def_end_char = 21;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 42, 13);

    // If let scopes
    go_to.def_line = 47;
    go_to.def_start_char = 38;
    go_to.def_end_char = 39;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 47, 47);

    // Shadowing
    go_to.def_line = 47;
    go_to.def_start_char = 8;
    go_to.def_end_char = 17;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 50, 29);

    // Variable type ascriptions
    go_to.def_line = 6;
    go_to.def_start_char = 5;
    go_to.def_end_char = 16;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 53, 21);

    // Complex type ascriptions
    go_to.def_line = 61;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    go_to.def_path = "fixtures-std/src/result.sw";
    lsp::definition_check_with_req_offset(&server, &mut go_to, 56, 22);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 31);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 60);
    go_to.def_line = 81;
    go_to.def_path = "fixtures-std/src/option.sw";
    lsp::definition_check_with_req_offset(&server, &mut go_to, 56, 28);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 39);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 68);

    // ContractCaller
    go_to.def_line = 15;
    go_to.def_start_char = 4;
    go_to.def_end_char = 11;
    go_to.def_path = uri.as_str();
    lsp::definition_check_with_req_offset(&server, &mut go_to, 60, 34);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 60, 50);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 61, 50);

    let _ = server.shutdown_server();
}

#[tokio::test]
async fn go_to_definition_for_consts() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/consts/src/main.sw"),
    )
    .await;

    // value: TyExpression
    let mut contract_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 9,
        req_char: 24,
        def_line: 8,
        def_start_char: 11,
        def_end_char: 21,
        def_path: "fixtures-std/src/contract_id.sw",
    };
    lsp::definition_check(&server, &contract_go_to);

    contract_go_to.req_char = 34;
    contract_go_to.def_line = 40;
    contract_go_to.def_start_char = 7;
    contract_go_to.def_end_char = 11;
    lsp::definition_check(&server, &contract_go_to);

    // Constants defined in the same module
    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 20,
        req_char: 34,
        def_line: 6,
        def_start_char: 6,
        def_end_char: 16,
        def_path: uri.as_str(),
    };
    lsp::definition_check(&server, &contract_go_to);

    go_to.def_line = 9;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 21, 29);

    // Constants defined in a different module
    go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 73,
        def_line: 12,
        def_start_char: 10,
        def_end_char: 20,
        def_path: "consts/src/more_consts.sw",
    };
    lsp::definition_check(&server, &go_to);

    go_to.def_line = 13;
    go_to.def_start_char = 10;
    go_to.def_end_char = 18;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 31);

    // Constants with type ascriptions
    go_to.def_line = 6;
    go_to.def_start_char = 5;
    go_to.def_end_char = 9;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 10, 17);

    // Complex type ascriptions
    go_to.def_line = 81;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    go_to.def_path = "fixtures-std/src/option.sw";
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 17);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 24);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 11, 38);
}

#[tokio::test]
async fn go_to_definition_for_functions() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/functions/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 8,
        req_char: 14,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 12,
        def_path: uri.as_str(),
    };
    // Return type
    lsp::definition_check(&server, &go_to);
    go_to.def_line = 23;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 42);
    go_to.def_line = 28;
    go_to.def_start_char = 9;
    go_to.def_end_char = 18;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 55);

    // Function parameters
    go_to.def_line = 2;
    go_to.def_start_char = 7;
    go_to.def_end_char = 12;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 13, 16);
    go_to.def_line = 23;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 18);
    go_to.def_line = 28;
    go_to.def_start_char = 9;
    go_to.def_end_char = 18;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 33, 28);

    // Functions expression
    go_to.def_line = 8;
    go_to.def_start_char = 3;
    go_to.def_end_char = 6;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 19, 13);
}

#[tokio::test]
async fn go_to_definition_for_structs() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/structs/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 8,
        def_line: 9,
        def_start_char: 19,
        def_end_char: 20,
        def_path: uri.as_str(),
    };
    // Type Params
    lsp::definition_check(&server, &go_to);
    go_to.def_line = 3;
    go_to.def_start_char = 5;
    go_to.def_end_char = 9;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 12, 8);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 13, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 14, 9);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 15, 16);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 15, 23);
    go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 16,
        req_char: 11,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "fixtures-std/src/option.sw",
    };
    // Type Params
    lsp::definition_check(&server, &go_to);

    // Call Path
    go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 16,
        def_line: 19,
        def_start_char: 7,
        def_end_char: 13,
        def_path: uri.as_str(),
    };
    lsp::definition_check(&server, &go_to);
}

#[tokio::test]
async fn go_to_definition_for_impls() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/impls/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 16,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 17,
        def_path: uri.as_str(),
    };
    // TestStruct
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 7, 33);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 8, 17);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 8, 27);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 7,
        req_char: 15,
        def_line: 4,
        def_start_char: 6,
        def_end_char: 15,
        def_path: uri.as_str(),
    };
    // TestTrait
    lsp::definition_check(&server, &go_to);
}

#[tokio::test]
async fn go_to_definition_for_where_clause() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/where_clause/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 8,
        def_line: 2,
        def_start_char: 6,
        def_end_char: 12,
        def_path: uri.as_str(),
    };
    // Trait1
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 7, 8);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 7,
        req_char: 17,
        def_line: 3,
        def_start_char: 6,
        def_end_char: 12,
        def_path: uri.as_str(),
    };
    // Trait2
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 4,
        def_line: 5,
        def_start_char: 7,
        def_end_char: 8,
        def_path: uri.as_str(),
    };
    // A
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 7,
        req_char: 4,
        def_line: 5,
        def_start_char: 10,
        def_end_char: 11,
        def_path: uri.as_str(),
    };
    // B
    lsp::definition_check(&server, &go_to);
}

#[tokio::test]
async fn go_to_definition_for_enums() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/enums/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 16,
        req_char: 16,
        def_line: 3,
        def_start_char: 7,
        def_end_char: 17,
        def_path: uri.as_str(),
    };
    // Type Params
    lsp::definition_check(&server, &go_to);
    go_to.def_line = 8;
    go_to.def_start_char = 5;
    go_to.def_end_char = 10;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 17, 15);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 18, 20);

    // Variants
    go_to.def_line = 9;
    go_to.def_start_char = 4;
    go_to.def_end_char = 7;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 24, 21);
    go_to.def_line = 20;
    go_to.def_start_char = 4;
    go_to.def_end_char = 10;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 31);

    // Call Path
    go_to.def_line = 15;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 23);
}

#[tokio::test]
async fn go_to_definition_for_abi() {
    let server = ServerState::default();
    let uri = open(&server, test_fixtures_dir().join("tokens/abi/src/main.sw")).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 29,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 12,
        def_path: uri.as_str(),
    };
    // Return type
    lsp::definition_check(&server, &go_to);

    // Abi name
    go_to.def_line = 5;
    go_to.def_start_char = 4;
    go_to.def_end_char = 14;
    lsp::definition_check_with_req_offset(&server, &mut go_to, 9, 11);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 16, 15);
}

#[tokio::test]
async fn go_to_definition_for_storage() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/storage/src/main.sw"),
    )
    .await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 9,
        def_line: 12,
        def_start_char: 0,
        def_end_char: 7,
        def_path: "sway-lsp/tests/fixtures/tokens/storage/src/main.sw",
    };
    // storage
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 8);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 26, 8);

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 17,
        def_line: 13,
        def_start_char: 4,
        def_end_char: 8,
        def_path: "sway-lsp/tests/fixtures/tokens/storage/src/main.sw",
    };
    // storage.var1
    lsp::definition_check(&server, &go_to);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 25, 17);
    lsp::definition_check_with_req_offset(&server, &mut go_to, 26, 17);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 21,
        def_line: 3,
        def_start_char: 4,
        def_end_char: 5,
        def_path: "sway-lsp/tests/fixtures/tokens/storage/src/main.sw",
    };
    // storage.var1.x
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 25,
        req_char: 21,
        def_line: 4,
        def_start_char: 4,
        def_end_char: 5,
        def_path: "sway-lsp/tests/fixtures/tokens/storage/src/main.sw",
    };
    // storage.var1.y
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 26,
        req_char: 21,
        def_line: 5,
        def_start_char: 4,
        def_end_char: 5,
        def_path: "sway-lsp/tests/fixtures/tokens/storage/src/main.sw",
    };
    // storage.var1.z
    lsp::definition_check(&server, &go_to);

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 26,
        req_char: 23,
        def_line: 9,
        def_start_char: 4,
        def_end_char: 5,
        def_path: "sway-lsp/tests/fixtures/tokens/storage/src/main.sw",
    };
    // storage.var1.z.x
    lsp::definition_check(&server, &go_to);

    let _ = server.shutdown_server();
}

//------------------- HOVER DOCUMENTATION -------------------//

#[tokio::test]
async fn hover_docs_for_consts() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/consts/src/main.sw"),
    )
    .await;

    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 20,
        req_char: 33,
        documentation: vec![" documentation for CONSTANT_1"],
    };

    lsp::hover_request(&server, &hover);
    hover.req_char = 49;
    hover.documentation = vec![" CONSTANT_2 has a value of 200"];
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_functions() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/functions/src/main.sw"),
    )
    .await;

    let hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 20,
        req_char: 14,
        documentation: vec!["```sway\npub fn bar(p: Point) -> Point\n```\n---\n A function declaration with struct as a function parameter\n\n---\nGo to [Point](command:sway.goToLocation?%5B%7B%22range%22%3A%7B%22end%22%3A%7B%22character%22%3A1%2C%22line%22%3A5%7D%2C%22start%22%3A%7B%22character%22%3A0%2C%22line%22%3A2%7D%7D%2C%22uri%22%3A%22file","sway%2Fsway-lsp%2Ftests%2Ffixtures%2Ftokens%2Ffunctions%2Fsrc%2Fmain.sw%22%7D%5D \"functions::Point\")"],
    };
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_structs() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/structs/src/main.sw"),
    )
    .await;
    let data_documention = "```sway\nenum Data\n```\n---\n My data enum";

    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 12,
        req_char: 10,
        documentation: vec![data_documention],
    };
    lsp::hover_request(&server, &hover);
    hover.req_line = 13;
    hover.req_char = 15;
    lsp::hover_request(&server, &hover);
    hover.req_line = 14;
    hover.req_char = 10;
    lsp::hover_request(&server, &hover);
    hover.req_line = 15;
    hover.req_char = 16;
    lsp::hover_request(&server, &hover);

    hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 9,
        req_char: 8,
        documentation: vec!["```sway\nstruct MyStruct\n```\n---\n My struct type"],
    };
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_enums() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/enums/src/main.sw"),
    )
    .await;

    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 16,
        req_char: 19,
        documentation: vec!["```sway\nstruct TestStruct\n```\n---\n Test Struct Docs"],
    };
    lsp::hover_request(&server, &hover);
    hover.req_line = 18;
    hover.req_char = 20;
    hover.documentation = vec!["```sway\nenum Color\n```\n---\n Color enum with RGB variants"];
    lsp::hover_request(&server, &hover);
    hover.req_line = 25;
    hover.req_char = 29;
    hover.documentation = vec![" Docs for variants"];
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_abis() {
    let server = ServerState::default();
    let uri = open(&server, test_fixtures_dir().join("tokens/abi/src/main.sw")).await;

    let hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 16,
        req_char: 14,
        documentation: vec!["```sway\nabi MyContract\n```\n---\n Docs for MyContract"],
    };
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_variables() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/variables/src/main.sw"),
    )
    .await;

    let hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 60,
        req_char: 14,
        documentation: vec!["```sway\nlet variable8: ContractCaller<TestAbi>\n```\n---"],
    };
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_with_code_examples() {
    let server = ServerState::default();
    let uri = open(&server, doc_comments_dir().join("src/main.sw")).await;

    let hover = HoverDocumentation {
            req_uri: &uri,
            req_line: 44,
            req_char: 24,
            documentation: vec!["```sway\nstruct Data\n```\n---\n Struct holding:\n\n 1. A `value` of type `NumberOrString`\n 2. An `address` of type `u64`"],
        };
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_self_keywords() {
    let server = ServerState::default();
    let uri = open(&server, test_fixtures_dir().join("completion/src/main.sw")).await;

    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 11,
        req_char: 13,
        documentation: vec!["\n```sway\nself\n```\n\n---\n\n The receiver of a method, or the current module.\n\n `self` is used in two situations: referencing the current module and marking\n the receiver of a method.\n\n In paths, `self` can be used to refer to the current module, either in a\n [`use`] statement or in a path to access an element:\n\n ```sway\n use std::contract_id::{self, ContractId};\n ```\n\n Is functionally the same as:\n\n ```sway\n use std::contract_id;\n use std::contract_id::ContractId;\n ```\n\n `self` as the current receiver for a method allows to omit the parameter\n type most of the time. With the exception of this particularity, `self` is\n used much like any other parameter:\n\n ```sway\n struct Foo(u32);\n\n impl Foo {\n     // No `self`.\n     fn new() -> Self {\n         Self(0)\n     }\n\n     // Borrowing `self`.\n     fn value(&self) -> u32 {\n         self.0\n     }\n\n     // Updating `self` mutably.\n     fn clear(ref mut self) {\n         self.0 = 0\n     }\n }\n ```"],
    };

    lsp::hover_request(&server, &hover);
    hover.req_char = 24;
    hover.documentation = vec!["```sway\nstruct MyStruct\n```\n---\n\n---\n[2 implementations](command:sway.peekLocations?%5B%7B%22locations%22%3A%5B%7B%22range%22%3A%7B%22end%22%3A%7B%22character%22%3A1%2C%22line%22%3A4%7D%2C%22start%22%3A%7B%22character%22%3A0%2C%22line%22%3A2%7D%7D%2C%22uri%22%3A%22file","sway%2Fsway-lsp%2Ftests%2Ffixtures%2Fcompletion%2Fsrc%2Fmain.sw%22%7D%2C%7B%22range%22%3A%7B%22end%22%3A%7B%22character%22%3A1%2C%22line%22%3A14%7D%2C%22start%22%3A%7B%22character%22%3A0%2C%22line%22%3A6%7D%7D%2C%22uri%22%3A%22file","sway%2Fsway-lsp%2Ftests%2Ffixtures%2Fcompletion%2Fsrc%2Fmain.sw%22%7D%5D%7D%5D \"Go to implementations\")"];
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn hover_docs_for_boolean_keywords() {
    let server = ServerState::default();
    let uri = open(
        &server,
        test_fixtures_dir().join("tokens/storage/src/main.sw"),
    )
    .await;

    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 13,
        req_char: 36,
        documentation: vec!["\n```sway\nfalse\n```\n\n---\n\n A value of type [`bool`] representing logical **false**.\n\n `false` is the logical opposite of [`true`].\n\n See the documentation for [`true`] for more information."],
    };

    lsp::hover_request(&server, &hover);
    hover.req_line = 25;
    hover.req_char = 31;
    hover.documentation = vec!["\n```sway\ntrue\n```\n\n---\n\n A value of type [`bool`] representing logical **true**.\n\n Logically `true` is not equal to [`false`].\n\n ## Control structures that check for **true**\n\n Several of Sway's control structures will check for a `bool` condition evaluating to **true**.\n\n   * The condition in an [`if`] expression must be of type `bool`.\n     Whenever that condition evaluates to **true**, the `if` expression takes\n     on the value of the first block. If however, the condition evaluates\n     to `false`, the expression takes on value of the `else` block if there is one.\n\n   * [`while`] is another control flow construct expecting a `bool`-typed condition.\n     As long as the condition evaluates to **true**, the `while` loop will continually\n     evaluate its associated block.\n\n   * [`match`] arms can have guard clauses on them."];
    lsp::hover_request(&server, &hover);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn rename() {
    let server = ServerState::default();
    let uri = open(&server, test_fixtures_dir().join("renaming/src/main.sw")).await;

    // Struct expression variable
    let rename = Rename {
        req_uri: &uri,
        req_line: 24,
        req_char: 19,
        new_name: "pnt", // from "point"
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // Enum
    let rename = Rename {
        req_uri: &uri,
        req_line: 21,
        req_char: 17,
        new_name: "MyEnum", // from "Color"
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // Enum Variant
    let rename = Rename {
        req_uri: &uri,
        req_line: 21,
        req_char: 20,
        new_name: "Pink", // from "Red"
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // raw identifier syntax
    let rename = Rename {
        req_uri: &uri,
        req_line: 28,
        req_char: 16,
        new_name: "new_var_name", // from r#struct
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // Function name defined in external module
    let rename = Rename {
        req_uri: &uri,
        req_line: 33,
        req_char: 25,
        new_name: "better_func_name", // from test_fun
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // Function method in ABI declaration
    let rename = Rename {
        req_uri: &uri,
        req_line: 41,
        req_char: 16,
        new_name: "name_func_name", // from test_function
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // Function method in ABI implementation
    let rename = Rename {
        req_uri: &uri,
        req_line: 45,
        req_char: 16,
        new_name: "name_func_name", // from test_function
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let _ = lsp::rename_request(&server, &rename);

    // Type alias used in function call
    let rename = Rename {
        req_uri: &uri,
        req_line: 55,
        req_char: 8,
        new_name: "Alias11", // from Alias1
    };
    let _ = lsp::prepare_rename_request(&server, &rename);
    let result = lsp::rename_request(&server, &rename);
    assert_eq!(result.changes.unwrap().values().next().unwrap().len(), 3);

    // Fail to rename keyword
    let rename = Rename {
        req_uri: &uri,
        req_line: 11,
        req_char: 2,
        new_name: "StruCt", // from struct
    };
    assert_eq!(lsp::prepare_rename_request(&server, &rename), None);

    // Fail to rename module
    let rename = Rename {
        req_uri: &uri,
        req_line: 36,
        req_char: 13,
        new_name: "new_mod_name", // from std
    };
    assert_eq!(lsp::prepare_rename_request(&server, &rename), None);

    // Fail to rename a type defined in a module outside of the users workspace
    let rename = Rename {
        req_uri: &uri,
        req_line: 36,
        req_char: 33,
        new_name: "NEW_TYPE_NAME", // from ZERO_B256
    };
    assert_eq!(lsp::prepare_rename_request(&server, &rename), None);
    let _ = server.shutdown_server();
}

#[tokio::test]
async fn publish_diagnostics_dead_code_warning() {
    let (mut service, socket) = LspService::new(ServerState::new);
    let fixture = get_fixture(test_fixtures_dir().join("diagnostics/dead_code/expected.json"));
    let expected_requests = vec![fixture];
    let socket_handle = assert_server_requests(socket, expected_requests).await;
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

#[tokio::test]
async fn publish_diagnostics_multi_file() {
    let (mut service, socket) = LspService::new(ServerState::new);
    let fixture = get_fixture(test_fixtures_dir().join("diagnostics/multi_file/expected.json"));
    let expected_requests = vec![fixture];
    let socket_handle = assert_server_requests(socket, expected_requests).await;
    let _ = init_and_open(
        &mut service,
        test_fixtures_dir().join("diagnostics/multi_file/src/main.sw"),
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
        let server = ServerState::default();
        let uri = open(&server, $entry_point).await;

        // Call the specific LSP capability function that was passed in.
        let _ = $capability(&server, &uri);
        let _ = server.shutdown_server();
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
    lsp::semantic_tokens_request,
    doc_comments_dir().join("src/main.sw")
);
lsp_capability_test!(
    document_symbol,
    lsp::document_symbol_request,
    doc_comments_dir().join("src/main.sw")
);
lsp_capability_test!(
    format,
    lsp::format_request,
    doc_comments_dir().join("src/main.sw")
);
lsp_capability_test!(
    highlight,
    lsp::highlight_request,
    doc_comments_dir().join("src/main.sw")
);
lsp_capability_test!(
    code_action_abi,
    code_actions::code_action_abi_request,
    doc_comments_dir().join("src/main.sw")
);
lsp_capability_test!(
    code_action_function,
    code_actions::code_action_function_request,
    test_fixtures_dir().join("tokens/consts/src/main.sw")
);
lsp_capability_test!(
    code_action_trait_fn_request,
    code_actions::code_action_trait_fn_request,
    test_fixtures_dir().join("tokens/abi/src/main.sw")
);
lsp_capability_test!(
    code_action_struct,
    code_actions::code_action_struct_request,
    doc_comments_dir().join("src/main.sw")
);
lsp_capability_test!(
    code_action_struct_type_params,
    code_actions::code_action_struct_type_params_request,
    generic_impl_self_dir().join("src/main.sw")
);
lsp_capability_test!(
    code_action_struct_existing_impl,
    code_actions::code_action_struct_existing_impl_request,
    self_impl_reassignment_dir().join("src/main.sw")
);
lsp_capability_test!(
    code_action_auto_import_struct,
    code_actions::code_action_auto_import_struct_request,
    test_fixtures_dir().join("auto_import/src/main.sw")
);
lsp_capability_test!(
    code_action_auto_import_enum,
    code_actions::code_action_auto_import_enum_request,
    test_fixtures_dir().join("auto_import/src/main.sw")
);
lsp_capability_test!(
    code_action_auto_import_function,
    code_actions::code_action_auto_import_function_request,
    test_fixtures_dir().join("auto_import/src/main.sw")
);
lsp_capability_test!(
    code_action_auto_import_constant,
    code_actions::code_action_auto_import_constant_request,
    test_fixtures_dir().join("auto_import/src/main.sw")
);
lsp_capability_test!(
    code_action_auto_import_trait,
    code_actions::code_action_auto_import_trait_request,
    test_fixtures_dir().join("auto_import/src/main.sw")
);
lsp_capability_test!(
    code_action_auto_import_alias,
    code_actions::code_action_auto_import_alias_request,
    test_fixtures_dir().join("auto_import/src/main.sw")
);
lsp_capability_test!(
    code_lens,
    lsp::code_lens_request,
    runnables_test_dir().join("src/main.sw")
);
lsp_capability_test!(
    code_lens_empty,
    lsp::code_lens_empty_request,
    runnables_test_dir().join("src/other.sw")
);
lsp_capability_test!(
    completion,
    lsp::completion_request,
    test_fixtures_dir().join("completion/src/main.sw")
);

// This method iterates over all of the examples in the e2e langauge should_pass dir
// and saves the lexed, parsed, and typed ASTs to the users home directory.
// This makes it easy to grep for certain compiler types to inspect their use cases,
// providing necessary context when working on the traversal modules.
#[allow(unused)]
// #[tokio::test]
async fn write_all_example_asts() {
    let (mut service, _) = LspService::build(ServerState::new)
        .custom_method("sway/show_ast", ServerState::show_ast)
        .finish();
    let _ = lsp::initialize_request(&mut service).await;
    lsp::initialized_notification(&mut service).await;

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

    let server = ServerState::default();

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

            let uri = open(&server, manifest_dir.join("src/main.sw")).await;
            let example_dir = Some(Url::from_file_path(example_dir).unwrap());
            lsp::show_ast_request(&server, &uri, "lexed", example_dir.clone()).await;
            lsp::show_ast_request(&server, &uri, "parsed", example_dir.clone()).await;
            lsp::show_ast_request(&server, &uri, "typed", example_dir).await;
        }
    }
    let _ = server.shutdown_server();
}
