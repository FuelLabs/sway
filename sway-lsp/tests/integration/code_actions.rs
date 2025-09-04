//! This file contains methods used for simulating LSP code action json-rpc notifications and requests.
//! The methods are used to build and send requests and notifications to the LSP service
//! and assert the expected responses.

use lsp_types::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::HashMap;
use sway_lsp::{
    capabilities::diagnostic::DiagnosticData, handlers::request, server_state::ServerState,
};

fn create_code_action(
    uri: Url,
    title: String,
    changes: HashMap<Url, Vec<TextEdit>>,
    disabled: Option<CodeActionDisabled>,
    kind: Option<CodeActionKind>,
) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind,
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: None,
        disabled,
        data: Some(serde_json::to_value(uri).unwrap()),
    })
}

fn create_code_action_params(
    uri: Url,
    range: Range,
    diagnostics: Option<Vec<Diagnostic>>,
) -> CodeActionParams {
    CodeActionParams {
        text_document: TextDocumentIdentifier { uri },
        range,
        context: CodeActionContext {
            diagnostics: diagnostics.unwrap_or_default(),
            only: None,
            trigger_kind: Some(CodeActionTriggerKind::AUTOMATIC),
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    }
}

fn create_diagnostic_from_data(range: Range, data: DiagnosticData) -> Option<Vec<Diagnostic>> {
    Some(vec![Diagnostic {
        range,
        data: Some(json!(data)),
        ..Default::default()
    }])
}

fn create_changes_map(uri: &Url, range: Range, new_text: &str) -> HashMap<Url, Vec<TextEdit>> {
    HashMap::from([(
        uri.clone(),
        vec![TextEdit {
            range,
            new_text: new_text.to_string(),
        }],
    )])
}

fn create_changes_for_import(
    uri: &Url,
    line: u32,
    end_char: u32,
    call_path: &str,
    newline: &str,
) -> HashMap<Url, Vec<TextEdit>> {
    let range = Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: end_char,
        },
    };
    create_changes_map(uri, range, &format!("use {call_path};{newline}"))
}

fn create_changes_for_qualify(
    uri: &Url,
    line: u32,
    start_char: u32,
    end_char: u32,
    call_path: &str,
) -> HashMap<Url, Vec<TextEdit>> {
    let range = Range {
        start: Position {
            line,
            character: start_char,
        },
        end: Position {
            line,
            character: end_char,
        },
    };
    create_changes_map(uri, range, call_path)
}

async fn send_request(server: &ServerState, params: &CodeActionParams) -> Vec<CodeActionOrCommand> {
    request::handle_code_action(server, params.clone())
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("Empty response from server for request: {params:#?}"))
}

pub(crate) async fn code_action_abi_request(server: &ServerState, uri: &Url) {
    let params = create_code_action_params(
        uri.clone(),
        Range {
            start: Position {
                line: 27,
                character: 4,
            },
            end: Position {
                line: 27,
                character: 9,
            },
        },
        None,
    );

    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 31,
                character: 0,
            },
            end: Position {
                line: 31,
                character: 0,
            },
        },
        "\nimpl FooABI for Contract {\n    fn main() -> u64 {}\n}\n",
    );
    let expected = vec![create_code_action(
        uri.clone(),
        "Generate impl for `FooABI`".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    )];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_function_request(server: &ServerState, uri: &Url) {
    let params = create_code_action_params(
        uri.clone(),
        Range {
            start: Position {
                line: 18,
                character: 4,
            },
            end: Position {
                line: 18,
                character: 4,
            },
        },
        None,
    );

    let changes = create_changes_map(uri, Range {
            start: Position {
                line: 18,
                character: 0,
            },
            end: Position {
                line: 18,
                character: 0,
            },
        },
         "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n/// \n/// ### Reverts\n/// \n/// * List any cases where the function will revert\n/// \n/// ### Number of Storage Accesses\n/// \n/// * Reads: `0`\n/// * Writes: `0`\n/// * Clears: `0`\n/// \n/// ### Examples\n/// \n/// ```sway\n/// let x = test();\n/// ```\n");
    let expected = vec![create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    )];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_trait_fn_request(server: &ServerState, uri: &Url) {
    let params = create_code_action_params(
        uri.clone(),
        Range {
            start: Position {
                line: 10,
                character: 10,
            },
            end: Position {
                line: 10,
                character: 10,
            },
        },
        None,
    );

    let changes = create_changes_map(uri, Range {
          start: Position {
              line: 10,
              character: 0,
          },
          end: Position {
              line: 10,
              character: 0,
          },
        },
        "    /// Add a brief description.\n    /// \n    /// ### Additional Information\n    /// \n    /// Provide information beyond the core purpose or functionality.\n    /// \n    /// ### Returns\n    /// \n    /// * [Empty] - Add description here\n    /// \n    /// ### Reverts\n    /// \n    /// * List any cases where the function will revert\n    /// \n    /// ### Number of Storage Accesses\n    /// \n    /// * Reads: `0`\n    /// * Writes: `0`\n    /// * Clears: `0`\n    /// \n    /// ### Examples\n    /// \n    /// ```sway\n    /// let x = test_function();\n    /// ```\n");
    let expected = vec![create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    )];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_struct_request(server: &ServerState, uri: &Url) {
    let params = create_code_action_params(
        uri.clone(),
        Range {
            start: Position {
                line: 19,
                character: 11,
            },
            end: Position {
                line: 19,
                character: 11,
            },
        },
        None,
    );
    let mut expected: Vec<_> = Vec::new();

    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 25,
                character: 0,
            },
            end: Position {
                line: 25,
                character: 0,
            },
        },
        "\nimpl Data {\n    \n}\n",
    );
    expected.push(create_code_action(
        uri.clone(),
        "Generate impl for `Data`".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));

    let changes = create_changes_map(uri, Range {
              start: Position {
                  line: 25,
                  character: 0,
              },
              end: Position {
                  line: 25,
                  character: 0,
              },
          },
           "\nimpl Data {\n    fn new(value: NumberOrString, address: u64) -> Self { Self { value, address } }\n}\n");
    expected.push(create_code_action(
        uri.clone(),
        "Generate `new`".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));
    let changes = create_changes_map(
        uri,
        Range {
              start: Position {
                  line: 19,
                  character: 0,
              },
              end: Position {
                  line: 19,
                  character: 0,
              },
          },
           "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n");
    expected.push(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_struct_type_params_request(server: &ServerState, uri: &Url) {
    let params = create_code_action_params(
        uri.clone(),
        Range {
            start: Position {
                line: 4,
                character: 9,
            },
            end: Position {
                line: 4,
                character: 9,
            },
        },
        None,
    );
    let mut expected: Vec<_> = Vec::new();
    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 7,
                character: 0,
            },
            end: Position {
                line: 7,
                character: 0,
            },
        },
        "\nimpl<T> Data<T> {\n    \n}\n",
    );
    expected.push(create_code_action(
        uri.clone(),
        "Generate impl for `Data`".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));

    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 9,
                character: 0,
            },
            end: Position {
                line: 9,
                character: 0,
            },
        },
        "    fn new(value: T) -> Self { Self { value } }\n",
    );
    expected.push(create_code_action(
        uri.clone(),
        "Generate `new`".to_string(),
        changes,
        Some(CodeActionDisabled {
            reason: "Struct Data already has a `new` function".to_string(),
        }),
        Some(CodeActionKind::REFACTOR),
    ));

    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 4,
                character: 0,
            },
            end: Position {
                line: 4,
                character: 0,
            },
        },
        "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n");

    expected.push(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_struct_existing_impl_request(server: &ServerState, uri: &Url) {
    let params = create_code_action_params(
        uri.clone(),
        Range {
            start: Position {
                line: 2,
                character: 7,
            },
            end: Position {
                line: 2,
                character: 7,
            },
        },
        None,
    );
    let mut expected: Vec<_> = Vec::new();
    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 6,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 0,
            },
        },
        "\nimpl A {\n    \n}\n",
    );
    expected.push(create_code_action(
        uri.clone(),
        "Generate impl for `A`".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));

    let changes = create_changes_map(
        uri,
        Range {
            start: Position {
                line: 6,
                character: 0,
            },
            end: Position {
                line: 6,
                character: 0,
            },
        },
        "\nimpl A {\n    fn new(a: u64, b: u64) -> Self { Self { a, b } }\n}\n",
    );
    expected.push(create_code_action(
        uri.clone(),
        "Generate `new`".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));
    let changes = create_changes_map(
        uri,
        Range {
              start: Position {
                  line: 2,
                  character: 0,
              },
              end: Position {
                  line: 2,
                  character: 0,
              },
          },
        "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n");
    expected.push(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
        Some(CodeActionKind::REFACTOR),
    ));

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_auto_import_struct_request(server: &ServerState, uri: &Url) {
    // EvmAddress: external library
    let range = Range {
        start: Position {
            line: 8,
            character: 12,
        },
        end: Position {
            line: 8,
            character: 22,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("EvmAddress".to_string()),
            },
        ),
    );
    let call_path = "std::vm::evm::evm_address::EvmAddress";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 8, 12, 22, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);

    // DeepStruct: local library
    let range = Range {
        start: Position {
            line: 17,
            character: 12,
        },
        end: Position {
            line: 17,
            character: 22,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("DeepStruct".to_string()),
            },
        ),
    );
    let call_path = "deep_mod::deeper_mod::DeepStruct";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 17, 12, 22, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_auto_import_enum_request(server: &ServerState, uri: &Url) {
    // TODO: Add a test for an enum variant when https://github.com/FuelLabs/sway/issues/5188 is fixed.

    // AuthError: external library
    let range = Range {
        start: Position {
            line: 23,
            character: 28,
        },
        end: Position {
            line: 23,
            character: 37,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("AuthError".to_string()),
            },
        ),
    );
    let call_path = "std::auth::AuthError";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 23, 28, 37, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);

    // DeepEnum: local library
    let range = Range {
        start: Position {
            line: 16,
            character: 11,
        },
        end: Position {
            line: 16,
            character: 19,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("DeepEnum".to_string()),
            },
        ),
    );
    let call_path = "deep_mod::deeper_mod::DeepEnum";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 16, 11, 19, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_auto_import_function_request(server: &ServerState, uri: &Url) {
    // TODO: external library, test with `overflow``
    // Tracking issue: https://github.com/FuelLabs/sway/issues/5191

    // deep_fun: local library
    let range = Range {
        start: Position {
            line: 13,
            character: 4,
        },
        end: Position {
            line: 13,
            character: 12,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("deep_fun".to_string()),
            },
        ),
    );
    let call_path = "deep_mod::deeper_mod::deep_fun";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 13, 4, 12, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_auto_import_constant_request(server: &ServerState, uri: &Url) {
    // TODO: external library, test with ZERO_B256
    // Tracking issue: https://github.com/FuelLabs/sway/issues/5192

    // TEST_CONST: import a constant from a local library
    let range = Range {
        start: Position {
            line: 19,
            character: 12,
        },
        end: Position {
            line: 19,
            character: 22,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("TEST_CONST".to_string()),
            },
        ),
    );
    let call_path = "test_mod::TEST_CONST";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 23, "test_mod::{TEST_CONST, test_fun}", ""),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 19, 12, 22, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_auto_import_trait_request(server: &ServerState, uri: &Url) {
    // TryFrom: external library
    let range = Range {
        start: Position {
            line: 34,
            character: 5,
        },
        end: Position {
            line: 34,
            character: 12,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("TryFrom".to_string()),
            },
        ),
    );
    let call_path = "std::convert::TryFrom";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 34, 5, 12, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);

    // DeepTrait: local library
    let range = Range {
        start: Position {
            line: 30,
            character: 5,
        },
        end: Position {
            line: 30,
            character: 14,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("DeepTrait".to_string()),
            },
        ),
    );
    let call_path = "deep_mod::deeper_mod::DeepTrait";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path}`"),
            create_changes_for_import(uri, 5, 0, call_path, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path}`"),
            create_changes_for_qualify(uri, 30, 5, 14, call_path),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}

pub(crate) async fn code_action_auto_import_alias_request(server: &ServerState, uri: &Url) {
    // TODO: find an example in an external library
    // A: local library with multiple possible imports
    let range = Range {
        start: Position {
            line: 14,
            character: 4,
        },
        end: Position {
            line: 14,
            character: 5,
        },
    };

    let params = create_code_action_params(
        uri.clone(),
        range,
        create_diagnostic_from_data(
            range,
            DiagnosticData {
                unknown_symbol_name: Some("A".to_string()),
            },
        ),
    );
    let call_path1 = "deep_mod::deeper_mod::A";
    let call_path2 = "test_mod::A";
    let expected = vec![
        create_code_action(
            uri.clone(),
            format!("Import `{call_path1}`"),
            create_changes_for_import(uri, 5, 0, call_path1, "\n"),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Import `{call_path2}`"),
            create_changes_for_import(uri, 5, 23, "test_mod::{A, test_fun}", ""),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path1}`"),
            create_changes_for_qualify(uri, 14, 4, 5, call_path1),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
        create_code_action(
            uri.clone(),
            format!("Qualify as `{call_path2}`"),
            create_changes_for_qualify(uri, 14, 4, 5, call_path2),
            None,
            Some(CodeActionKind::QUICKFIX),
        ),
    ];

    let actual = send_request(server, &params).await;
    assert_eq!(expected, actual);
}
