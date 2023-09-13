//! This file contains methods used for simulating LSP code action json-rpc notifications and requests.
//! The methods are used to build and send requests and notifications to the LSP service
//! and assert the expected responses.

use lsp_types::*;
use std::collections::HashMap;
use sway_lsp::{handlers::request, server_state::ServerState};

fn create_code_action(
    uri: Url,
    title: String,
    changes: HashMap<Url, Vec<TextEdit>>,
    disabled: Option<CodeActionDisabled>,
) -> CodeAction {
    CodeAction {
        title,
        kind: Some(CodeActionKind::REFACTOR),
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
    }
}

fn create_code_action_params(uri: Url, range: Range) -> CodeActionParams {
    CodeActionParams {
        text_document: TextDocumentIdentifier { uri },
        range,
        context: CodeActionContext {
            diagnostics: vec![],
            only: None,
            trigger_kind: Some(CodeActionTriggerKind::AUTOMATIC),
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    }
}

pub(crate) fn code_action_abi_request(server: &ServerState, uri: &Url) {
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
    );
    let res = request::handle_code_action(server, params);
    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 31,
                    character: 0,
                },
                end: Position {
                    line: 31,
                    character: 0,
                },
            },
            new_text: "\nimpl FooABI for Contract {\n    fn main() -> u64 {}\n}\n".to_string(),
        }],
    );
    let expected = vec![CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate impl for `FooABI`".to_string(),
        changes,
        None,
    ))];
    assert_eq!(res.unwrap().unwrap(), expected);
}

pub(crate) fn code_action_function_request(server: &ServerState, uri: &Url) {
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
    );
    let res = request::handle_code_action(server, params);
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![TextEdit {
    range: Range {
        start: Position {
            line: 18,
            character: 0,
        },
        end: Position {
            line: 18,
            character: 0,
        },
      },
      new_text: "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n/// \n/// ### Reverts\n/// \n/// * List any cases where the function will revert\n/// \n/// ### Number of Storage Accesses\n/// \n/// * Reads: `0`\n/// * Writes: `0`\n/// * Clears: `0`\n/// \n/// ### Examples\n/// \n/// ```sway\n/// let x = test();\n/// ```\n".to_string(),
    }]);
    let expected = vec![CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
    ))];
    assert_eq!(res.unwrap().unwrap(), expected);
}

pub(crate) fn code_action_trait_fn_request(server: &ServerState, uri: &Url) {
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
    );
    let res = request::handle_code_action(server, params);
    let mut changes = HashMap::new();

    changes.insert(uri.clone(), vec![TextEdit {
      range: Range {
          start: Position {
              line: 10,
              character: 0,
          },
          end: Position {
              line: 10,
              character: 0,
          },
        },
        new_text: "    /// Add a brief description.\n    /// \n    /// ### Additional Information\n    /// \n    /// Provide information beyond the core purpose or functionality.\n    /// \n    /// ### Returns\n    /// \n    /// * [Empty] - Add description here\n    /// \n    /// ### Reverts\n    /// \n    /// * List any cases where the function will revert\n    /// \n    /// ### Number of Storage Accesses\n    /// \n    /// * Reads: `0`\n    /// * Writes: `0`\n    /// * Clears: `0`\n    /// \n    /// ### Examples\n    /// \n    /// ```sway\n    /// let x = test_function();\n    /// ```\n".to_string(),
      }]);
    let expected = vec![CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
    ))];
    assert_eq!(res.unwrap().unwrap(), expected);
}

pub(crate) fn code_action_struct_request(server: &ServerState, uri: &Url) {
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
    );
    let res = request::handle_code_action(server, params);
    let mut expected: Vec<_> = Vec::new();
    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 25,
                    character: 0,
                },
                end: Position {
                    line: 25,
                    character: 0,
                },
            },
            new_text: "\nimpl Data {\n    \n}\n".to_string(),
        }],
    );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate impl for `Data`".to_string(),
        changes,
        None,
    )));
    let mut changes = HashMap::new();
    changes.insert(
      uri.clone(),
      vec![TextEdit {
          range: Range {
              start: Position {
                  line: 25,
                  character: 0,
              },
              end: Position {
                  line: 25,
                  character: 0,
              },
          },
          new_text: "\nimpl Data {\n    fn new(value: NumberOrString, address: u64) -> Self { Self { value, address } }\n}\n".to_string(),
      }],
  );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate `new`".to_string(),
        changes,
        None,
    )));
    let mut changes = HashMap::new();
    changes.insert(
      uri.clone(),
      vec![TextEdit {
          range: Range {
              start: Position {
                  line: 19,
                  character: 0,
              },
              end: Position {
                  line: 19,
                  character: 0,
              },
          },
          new_text: "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n".to_string(),
      }],
  );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
    )));
    assert_eq!(res.unwrap().unwrap(), expected);
}

pub(crate) fn code_action_struct_type_params_request(server: &ServerState, uri: &Url) {
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
    );
    let res = request::handle_code_action(server, params);
    let mut expected: Vec<_> = Vec::new();
    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 7,
                    character: 0,
                },
                end: Position {
                    line: 7,
                    character: 0,
                },
            },
            new_text: "\nimpl<T> Data<T> {\n    \n}\n".to_string(),
        }],
    );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate impl for `Data`".to_string(),
        changes,
        None,
    )));

    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 9,
                    character: 0,
                },
                end: Position {
                    line: 9,
                    character: 0,
                },
            },
            new_text: "    fn new(value: T) -> Self { Self { value } }\n".to_string(),
        }],
    );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate `new`".to_string(),
        changes,
        Some(CodeActionDisabled {
            reason: "Struct Data already has a `new` function".to_string(),
        }),
    )));

    let mut changes = HashMap::new();
    changes.insert(
    uri.clone(),
    vec![TextEdit {
        range: Range {
            start: Position {
                line: 4,
                character: 0,
            },
            end: Position {
                line: 4,
                character: 0,
            },
        },
        new_text: "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n".to_string(),
    }],
);
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
    )));
    assert_eq!(res.unwrap().unwrap(), expected);
}

pub(crate) fn code_action_struct_existing_impl_request(server: &ServerState, uri: &Url) {
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
    );
    let res = request::handle_code_action(server, params);
    let mut expected: Vec<_> = Vec::new();
    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 6,
                    character: 0,
                },
                end: Position {
                    line: 6,
                    character: 0,
                },
            },
            new_text: "\nimpl A {\n    \n}\n".to_string(),
        }],
    );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate impl for `A`".to_string(),
        changes,
        None,
    )));

    let mut changes = HashMap::new();
    changes.insert(
        uri.clone(),
        vec![TextEdit {
            range: Range {
                start: Position {
                    line: 8,
                    character: 0,
                },
                end: Position {
                    line: 8,
                    character: 0,
                },
            },
            new_text: "    fn new(a: u64, b: u64) -> Self { Self { a, b } }\n".to_string(),
        }],
    );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate `new`".to_string(),
        changes,
        None,
    )));

    let mut changes = HashMap::new();
    changes.insert(
      uri.clone(),
      vec![TextEdit {
          range: Range {
              start: Position {
                  line: 2,
                  character: 0,
              },
              end: Position {
                  line: 2,
                  character: 0,
              },
          },
          new_text: "/// Add a brief description.\n/// \n/// ### Additional Information\n/// \n/// Provide information beyond the core purpose or functionality.\n".to_string(),
      }],
  );
    expected.push(CodeActionOrCommand::CodeAction(create_code_action(
        uri.clone(),
        "Generate a documentation template".to_string(),
        changes,
        None,
    )));

    let result = res.unwrap().unwrap();
    assert_eq!(result, expected);
}
