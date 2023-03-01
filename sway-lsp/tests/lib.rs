pub mod integration;

use crate::integration::{code_actions, lsp};
use std::{fs, path::PathBuf};
use sway_lsp::server::Backend;
use sway_lsp_test_utils::{
    assert_server_requests, dir_contains_forc_manifest, doc_comments_dir, e2e_language_dir,
    e2e_test_dir, generic_impl_self_dir, get_fixture, load_sway_example, runnables_test_dir,
    self_impl_reassignment_dir, sway_workspace_dir, test_fixtures_dir,
};
use tower_lsp::{
    jsonrpc::{self, Response},
    lsp_types::*,
    LspService,
};

/// Holds the information needed to check the response of a goto definition request.
pub(crate) struct GotoDefinition<'a> {
    req_uri: &'a Url,
    req_line: i32,
    req_char: i32,
    def_line: i32,
    def_start_char: i32,
    def_end_char: i32,
    def_path: &'a str,
}

/// Contains data required to evaluate a hover request response.
pub(crate) struct HoverDocumentation<'a> {
    req_uri: &'a Url,
    req_line: i32,
    req_char: i32,
    documentation: &'a str,
}

async fn init_and_open(service: &mut LspService<Backend>, entry_point: PathBuf) -> Url {
    let _ = lsp::initialize_request(service).await;
    lsp::initialized_notification(service).await;
    let (uri, sway_program) = load_sway_example(entry_point);
    lsp::did_open_notification(service, &uri, &sway_program).await;
    uri
}

async fn shutdown_and_exit(service: &mut LspService<Backend>) {
    let _ = lsp::shutdown_request(service).await;
    lsp::exit_notification(service).await;
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
            lsp::did_open_notification(&mut service, &uri, &sway_program).await;
            let _ = lsp::show_ast_request(&mut service, &uri, "lexed", example_dir.clone()).await;
            let _ = lsp::show_ast_request(&mut service, &uri, "parsed", example_dir.clone()).await;
            let _ = lsp::show_ast_request(&mut service, &uri, "typed", example_dir).await;
        }
    }
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn initialize() {
    let (mut service, _) = LspService::new(Backend::new);
    let _ = lsp::initialize_request(&mut service).await;
}

#[tokio::test]
async fn initialized() {
    let (mut service, _) = LspService::new(Backend::new);
    let _ = lsp::initialize_request(&mut service).await;
    lsp::initialized_notification(&mut service).await;
}

#[tokio::test]
async fn initializes_only_once() {
    let (mut service, _) = LspService::new(Backend::new);
    let initialize = lsp::initialize_request(&mut service).await;
    lsp::initialized_notification(&mut service).await;
    let response = lsp::call_request(&mut service, initialize).await;
    let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
    assert_eq!(response, Ok(Some(err)));
}

#[tokio::test]
async fn shutdown() {
    let (mut service, _) = LspService::new(Backend::new);
    let _ = lsp::initialize_request(&mut service).await;
    lsp::initialized_notification(&mut service).await;
    let shutdown = lsp::shutdown_request(&mut service).await;
    let response = lsp::call_request(&mut service, shutdown).await;
    let err = Response::from_error(1.into(), jsonrpc::Error::invalid_request());
    assert_eq!(response, Ok(Some(err)));
    lsp::exit_notification(&mut service).await;
}

#[tokio::test]
async fn refuses_requests_after_shutdown() {
    let (mut service, _) = LspService::new(Backend::new);
    let _ = lsp::initialize_request(&mut service).await;
    let shutdown = lsp::shutdown_request(&mut service).await;
    let response = lsp::call_request(&mut service, shutdown).await;
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
    lsp::did_close_notification(&mut service).await;
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn did_change() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let _ = lsp::did_change_request(&mut service, &uri).await;
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn lsp_syncs_with_workspace_edits() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let mut i = 0..;
    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 44,
        req_char: 24,
        def_line: 19,
        def_start_char: 7,
        def_end_char: 11,
        def_path: uri.as_str(),
    };
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    let _ = lsp::did_change_request(&mut service, &uri).await;
    go_to.def_line = 20;
    definition_check_with_req_offset(&mut service, &mut go_to, 45, 24, &mut i).await;
    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn show_ast() {
    let (mut service, _) = LspService::build(Backend::new)
        .custom_method("sway/show_ast", Backend::show_ast)
        .finish();

    let uri = init_and_open(&mut service, e2e_test_dir().join("src/main.sw")).await;
    let _ = lsp::show_ast_request(&mut service, &uri, "typed", None).await;
    shutdown_and_exit(&mut service).await;
}

//------------------- GO TO DEFINITION -------------------//

#[tokio::test]
async fn go_to_definition() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(&mut service, doc_comments_dir().join("src/main.sw")).await;
    let mut i = 0..;
    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 44,
        req_char: 24,
        def_line: 19,
        def_start_char: 7,
        def_end_char: 11,
        def_path: uri.as_str(),
    };
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    shutdown_and_exit(&mut service).await;
}

async fn definition_check_with_req_offset<'a>(
    service: &mut LspService<Backend>,
    go_to: &mut GotoDefinition<'a>,
    req_line: i32,
    req_char: i32,
    ids: &mut impl Iterator<Item = i64>,
) {
    go_to.req_line = req_line;
    go_to.req_char = req_char;
    let _ = lsp::definition_check(service, go_to, ids).await;
}

#[tokio::test]
async fn go_to_definition_for_fields() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/fields/src/main.sw"),
    )
    .await;
    let mut i = 0..;

    let mut opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 5,
        req_char: 8,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "sway-lib-std/src/option.sw",
    };
    // Option
    let _ = lsp::definition_check(&mut service, &opt_go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 5, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 9, 9, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 9, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 13, 12, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 13, 19, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 13, 34, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 13, 47, &mut i).await;

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 10,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 11,
        def_path: "sway-lsp/tests/fixtures/tokens/fields/src/foo.sw",
    };
    // foo
    let _ = lsp::definition_check(&mut service, &opt_go_to, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &opt_go_to, &mut i).await;

    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn go_to_definition_inside_turbofish() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/turbofish/src/main.sw"),
    )
    .await;
    let mut i = 0..;

    let mut opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 15,
        req_char: 12,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "sway-lib-std/src/option.sw",
    };
    // option.sw
    let _ = lsp::definition_check(&mut service, &opt_go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 16, 17, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 17, 29, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 18, 19, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 20, 13, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 21, 19, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 22, 29, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 23, 18, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut opt_go_to, 24, 26, &mut i).await;

    let mut res_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 20,
        req_char: 19,
        def_line: 61,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "sway-lib-std/src/result.sw",
    };
    // result.sw
    let _ = lsp::definition_check(&mut service, &res_go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut res_go_to, 21, 25, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut res_go_to, 22, 36, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut res_go_to, 23, 27, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut res_go_to, 24, 33, &mut i).await;

    shutdown_and_exit(&mut service).await;
}

#[tokio::test]
async fn go_to_definition_for_matches() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/matches/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 19, 18, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 22, 18, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 22, 30, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 23, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 38, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 25,
        req_char: 19,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "sway-lib-std/src/option.sw",
    };
    // Option
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 33, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 26, 11, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 27, 11, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 27, 22, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 11, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 22, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 25,
        req_char: 27,
        def_line: 85,
        def_start_char: 4,
        def_end_char: 8,
        def_path: "sway-lib-std/src/option.sw",
    };
    // Some
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 27, 17, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 17, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 30, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 26,
        req_char: 17,
        def_line: 83,
        def_start_char: 4,
        def_end_char: 8,
        def_path: "sway-lib-std/src/option.sw",
    };
    // None
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 27, 30, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

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
    let mut i = 0..;

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 2,
        req_char: 6,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 16,
        def_path: "sway-lsp/tests/fixtures/tokens/modules/src/test_mod.sw",
    };
    // dep test_mod;
    let _ = lsp::definition_check(&mut service, &opt_go_to, &mut i).await;

    let opt_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 3,
        req_char: 6,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 15,
        def_path: "sway-lsp/tests/fixtures/tokens/modules/src/dir_mod/mod.sw",
    };
    // dep dir_mod/mod;
    let _ = lsp::definition_check(&mut service, &opt_go_to, &mut i).await;

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
    let mut i = 0..;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 13,
        def_line: 3,
        def_start_char: 8,
        def_end_char: 11,
        def_path: "sway-lib-std/src/lib.sw",
    };
    // std
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 12, 14, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 18, 5, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 24, 13, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 7, 5, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 19,
        def_line: 74,
        def_start_char: 8,
        def_end_char: 14,
        def_path: "sway-lib-std/src/option.sw",
    };
    // option
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 10,
        req_char: 27,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "sway-lib-std/src/option.sw",
    };
    // Option
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 14, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 17,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 10,
        def_path: "sway-lib-std/src/vm/mod.sw",
    };
    // vm
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 22,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 11,
        def_path: "sway-lib-std/src/vm/evm/mod.sw",
    };
    // evm
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 27,
        def_line: 1,
        def_start_char: 8,
        def_end_char: 19,
        def_path: "sway-lib-std/src/vm/evm/evm_address.sw",
    };
    // evm_address
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 12,
        req_char: 42,
        def_line: 7,
        def_start_char: 11,
        def_end_char: 21,
        def_path: "sway-lib-std/src/vm/evm/evm_address.sw",
    };
    // EvmAddress
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 16,
        req_char: 6,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 16,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // test_mod
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 22, 7, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 5, 5, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 8,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 16,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod.sw",
    };
    // deep_mod
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 6, 6, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 27, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 29, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 30, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 32, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 16, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 17,
        req_char: 18,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 18,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // deeper_mod
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 6, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 27, 28, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 28, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 29, 28, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 30, 28, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 32, 28, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 28, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 38, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 29, 38, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 30, 38, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 37, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 48, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 30, 48, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 6, 28, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 18,
        req_char: 11,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 14,
        def_path: "sway-lib-std/src/assert.sw",
    };
    // assert
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 19,
        req_char: 13,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 12,
        def_path: "sway-lib-core/src/lib.sw",
    };
    // core
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 19,
        req_char: 21,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 18,
        def_path: "sway-lib-core/src/primitives.sw",
    };
    // primitives
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 20, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 21,
        req_char: 4,
        def_line: 6,
        def_start_char: 5,
        def_end_char: 6,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/test_mod.sw",
    };
    // A impl
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 22, 14, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 22, 18, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 20,
        def_line: 0,
        def_start_char: 8,
        def_end_char: 17,
        def_path: "sway-lib-std/src/constants.sw",
    };
    // constants
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 7, 11, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 7, 23, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 24,
        req_char: 31,
        def_line: 5,
        def_start_char: 10,
        def_end_char: 19,
        def_path: "sway-lib-std/src/constants.sw",
    };
    // ZERO_B256
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 7, 31, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 19,
        req_char: 31,
        def_line: 2,
        def_start_char: 5,
        def_end_char: 8,
        def_path: "sway-lib-core/src/primitives.sw",
    };
    // u64
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    let mut go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 13,
        req_char: 17,
        def_line: 74,
        def_start_char: 5,
        def_end_char: 9,
        def_path: "sway-lib-core/src/primitives.sw",
    };
    // b256
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 31, &mut i).await;

    let go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 39,
        def_line: 2,
        def_start_char: 7,
        def_end_char: 15,
        def_path: "sway-lsp/tests/fixtures/tokens/paths/src/deep_mod/deeper_mod.sw",
    };
    // dfun
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

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
    let mut i = 0..;

    let mut trait_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 6,
        req_char: 10,
        def_line: 2,
        def_start_char: 10,
        def_end_char: 15,
        def_path: "sway-lsp/tests/fixtures/tokens/traits/src/traits.sw",
    };

    let _ = lsp::definition_check(&mut service, &trait_go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut trait_go_to, 7, 10, &mut i).await;
    definition_check_with_req_offset(
        &mut service,
        &mut trait_go_to,
        10,
        6,
        // don't increment id for next check
        &mut i.clone(),
    )
    .await;
    trait_go_to.req_line = 7;
    trait_go_to.req_char = 20;
    trait_go_to.def_line = 3;
    let _ = lsp::definition_check(&mut service, &trait_go_to, &mut i).await;

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
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    // Function arguments
    go_to.def_line = 20;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 35, &mut i).await;

    // Struct fields
    go_to.def_line = 19;
    definition_check_with_req_offset(&mut service, &mut go_to, 28, 45, &mut i).await;

    // Enum fields
    go_to.def_line = 19;
    definition_check_with_req_offset(&mut service, &mut go_to, 31, 39, &mut i).await;

    // Tuple elements
    go_to.def_line = 21;
    definition_check_with_req_offset(&mut service, &mut go_to, 34, 20, &mut i).await;

    // Array elements
    go_to.def_line = 22;
    definition_check_with_req_offset(&mut service, &mut go_to, 37, 20, &mut i).await;

    // Scoped declarations
    go_to.def_line = 41;
    go_to.def_start_char = 12;
    go_to.def_end_char = 21;
    definition_check_with_req_offset(&mut service, &mut go_to, 42, 13, &mut i).await;

    // If let scopes
    go_to.def_line = 47;
    go_to.def_start_char = 38;
    go_to.def_end_char = 39;
    definition_check_with_req_offset(&mut service, &mut go_to, 47, 47, &mut i).await;

    // Shadowing
    go_to.def_line = 47;
    go_to.def_start_char = 8;
    go_to.def_end_char = 17;
    definition_check_with_req_offset(&mut service, &mut go_to, 50, 29, &mut i).await;

    // Variable type ascriptions
    go_to.def_line = 6;
    go_to.def_start_char = 5;
    go_to.def_end_char = 16;
    definition_check_with_req_offset(&mut service, &mut go_to, 53, 21, &mut i).await;

    // Complex type ascriptions
    go_to.def_line = 61;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    go_to.def_path = "sway-lib-std/src/result.sw";
    definition_check_with_req_offset(&mut service, &mut go_to, 56, 22, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 31, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 60, &mut i).await;
    go_to.def_line = 81;
    go_to.def_path = "sway-lib-std/src/option.sw";
    definition_check_with_req_offset(&mut service, &mut go_to, 56, 28, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 39, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 68, &mut i).await;

    // ContractCaller
    go_to.def_line = 15;
    go_to.def_start_char = 4;
    go_to.def_end_char = 11;
    go_to.def_path = uri.as_str();
    definition_check_with_req_offset(&mut service, &mut go_to, 60, 34, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 60, 50, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 61, 50, &mut i).await;

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
    let mut i = 0..;

    // value: TyExpression
    let mut contract_go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 9,
        req_char: 24,
        def_line: 18,
        def_start_char: 5,
        def_end_char: 9,
        def_path: "sway-lib-std/src/contract_id.sw",
    };
    let _ = lsp::definition_check(&mut service, &contract_go_to, &mut i).await;

    contract_go_to.req_char = 34;
    contract_go_to.def_line = 19;
    contract_go_to.def_start_char = 7;
    contract_go_to.def_end_char = 11;
    let _ = lsp::definition_check(&mut service, &contract_go_to, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &contract_go_to, &mut i).await;

    go_to.def_line = 9;
    definition_check_with_req_offset(&mut service, &mut go_to, 21, 29, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    go_to.def_line = 13;
    go_to.def_start_char = 10;
    go_to.def_end_char = 18;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 31, &mut i).await;

    // Constants with type ascriptions
    go_to.def_line = 6;
    go_to.def_start_char = 5;
    go_to.def_end_char = 9;
    definition_check_with_req_offset(&mut service, &mut go_to, 10, 17, &mut i).await;

    // Complex type ascriptions
    go_to.def_line = 81;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    go_to.def_path = "sway-lib-std/src/option.sw";
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 17, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 24, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 11, 38, &mut i).await;
}

#[tokio::test]
async fn go_to_definition_for_functions() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/functions/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    go_to.def_line = 23;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 42, &mut i).await;
    go_to.def_line = 28;
    go_to.def_start_char = 9;
    go_to.def_end_char = 18;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 55, &mut i).await;

    // Function parameters
    go_to.def_line = 2;
    go_to.def_start_char = 7;
    go_to.def_end_char = 12;
    definition_check_with_req_offset(&mut service, &mut go_to, 13, 16, &mut i).await;
    go_to.def_line = 23;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 18, &mut i).await;
    go_to.def_line = 28;
    go_to.def_start_char = 9;
    go_to.def_end_char = 18;
    definition_check_with_req_offset(&mut service, &mut go_to, 33, 28, &mut i).await;

    // Functions expression
    go_to.def_line = 8;
    go_to.def_start_char = 3;
    go_to.def_end_char = 6;
    definition_check_with_req_offset(&mut service, &mut go_to, 19, 13, &mut i).await;
}

#[tokio::test]
async fn go_to_definition_for_structs() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/structs/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    go_to.def_line = 3;
    go_to.def_start_char = 5;
    go_to.def_end_char = 9;
    definition_check_with_req_offset(&mut service, &mut go_to, 12, 8, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 13, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 14, 9, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 15, 16, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 15, 23, &mut i).await;
    go_to = GotoDefinition {
        req_uri: &uri,
        req_line: 16,
        req_char: 11,
        def_line: 81,
        def_start_char: 9,
        def_end_char: 15,
        def_path: "sway-lib-std/src/option.sw",
    };
    // Type Params
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
}

#[tokio::test]
async fn go_to_definition_for_impls() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/impls/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 7, 33, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 8, 17, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 8, 27, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
}

#[tokio::test]
async fn go_to_definition_for_where_clause() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/where_clause/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 7, 8, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
}

#[tokio::test]
async fn go_to_definition_for_enums() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/enums/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;
    go_to.def_line = 8;
    go_to.def_start_char = 5;
    go_to.def_end_char = 10;
    definition_check_with_req_offset(&mut service, &mut go_to, 17, 15, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 18, 20, &mut i).await;

    // Variants
    go_to.def_line = 9;
    go_to.def_start_char = 4;
    go_to.def_end_char = 7;
    definition_check_with_req_offset(&mut service, &mut go_to, 24, 21, &mut i).await;
    go_to.def_line = 20;
    go_to.def_start_char = 4;
    go_to.def_end_char = 10;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 31, &mut i).await;

    // Call Path
    go_to.def_line = 15;
    go_to.def_start_char = 9;
    go_to.def_end_char = 15;
    definition_check_with_req_offset(&mut service, &mut go_to, 25, 23, &mut i).await;
}

#[tokio::test]
async fn go_to_definition_for_abi() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/abi/src/main.sw"),
    )
    .await;
    let mut i = 0..;

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
    let _ = lsp::definition_check(&mut service, &go_to, &mut i).await;

    // Abi name
    go_to.def_line = 5;
    go_to.def_start_char = 4;
    go_to.def_end_char = 14;
    definition_check_with_req_offset(&mut service, &mut go_to, 9, 11, &mut i).await;
    definition_check_with_req_offset(&mut service, &mut go_to, 16, 15, &mut i).await;
}

//------------------- HOVER DOCUMENTATION -------------------//

#[tokio::test]
async fn hover_docs_for_consts() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/consts/src/main.sw"),
    )
    .await;
    let mut i = 0..;

    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 20,
        req_char: 33,
        documentation: " documentation for CONSTANT_1",
    };

    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
    hover.req_char = 49;
    hover.documentation = " CONSTANT_2 has a value of 200";
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
}

#[tokio::test]
async fn hover_docs_for_functions() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/functions/src/main.sw"),
    )
    .await;

    let hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 20,
        req_char: 14,
        documentation: "```sway\npub fn bar(p: Point) -> Point\n```\n---\n A function declaration with struct as a function parameter",
    };
    let mut i = 0..;
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
}

#[tokio::test]
async fn hover_docs_for_structs() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/structs/src/main.sw"),
    )
    .await;

    let data_documention = "```sway\nenum Data\n```\n---\n My data enum";

    let mut i = 0..;
    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 12,
        req_char: 10,
        documentation: data_documention,
    };
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
    hover.req_line = 13;
    hover.req_char = 15;
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
    hover.req_line = 14;
    hover.req_char = 10;
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
    hover.req_line = 15;
    hover.req_char = 16;
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;

    hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 9,
        req_char: 8,
        documentation: "```sway\nstruct MyStruct\n```\n---\n My struct type",
    };
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
}

#[tokio::test]
async fn hover_docs_for_enums() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/enums/src/main.sw"),
    )
    .await;

    let mut i = 0..;
    let mut hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 16,
        req_char: 19,
        documentation: "```sway\nstruct TestStruct\n```\n---\n Test Struct Docs",
    };
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
    hover.req_line = 18;
    hover.req_char = 20;
    hover.documentation = "```sway\nenum Color\n```\n---\n Color enum with RGB variants";
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
    hover.req_line = 25;
    hover.req_char = 29;
    hover.documentation = " Docs for variants";
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
}

#[tokio::test]
async fn hover_docs_for_abis() {
    let (mut service, _) = LspService::new(Backend::new);
    let uri = init_and_open(
        &mut service,
        test_fixtures_dir().join("tokens/abi/src/main.sw"),
    )
    .await;

    let mut i = 0..;
    let hover = HoverDocumentation {
        req_uri: &uri,
        req_line: 16,
        req_char: 14,
        documentation: "```sway\nabi MyContract\n```\n---\n Docs for MyContract",
    };
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
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
    let mut i = 0..;
    let _ = lsp::hover_request(&mut service, &hover, &mut i).await;
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
    code_lens,
    lsp::code_lens_request,
    runnables_test_dir().join("src/main.sw")
);
