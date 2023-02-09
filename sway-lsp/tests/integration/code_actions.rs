//! This file contains methods used for simulating LSP code action json-rpc notifications and requests.
//! The methods are used to build and send requests and notifications to the LSP service
//! and assert the expected responses.

use crate::integration::lsp::{build_request_with_id, call_request};
use assert_json_diff::assert_json_eq;
use serde_json::json;
use sway_lsp::server::Backend;
use tower_lsp::{
    jsonrpc::{Request, Response},
    lsp_types::*,
    LspService,
};

pub(crate) async fn code_action_abi_request(
    service: &mut LspService<Backend>,
    uri: &Url,
) -> Request {
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
            "title": "Generate impl for `FooABI`"
        }]),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    code_action
}

pub(crate) async fn code_action_struct_request(
    service: &mut LspService<Backend>,
    uri: &Url,
) -> Request {
    let params = json!({
        "textDocument": {
            "uri": uri,
        },
        "range" : {
            "start": {
                "line": 19,
                "character": 11
            },
            "end": {
                "line": 19,
                "character": 11
            }
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
        json!([
          {
            "data": uri,
            "edit": {
              "changes": {
                uri_string.clone(): [
                  {
                    "newText": "\nimpl Data {\n    \n}\n",
                    "range": {
                      "end": {
                        "character": 0,
                        "line": 25
                      },
                      "start": {
                        "character": 0,
                        "line": 25
                      }
                    }
                  }
                ]
              }
            },
            "kind": "refactor",
            "title": "Generate impl for `Data`"
          },
          {
            "data": uri,
            "edit": {
              "changes": {
                  uri_string: [
                  {
                    "newText": "\nimpl Data {\n\n    /// Struct holding:\n    ///\n    /// 1. A `value` of type `NumberOrString`\n    /// 2. An `address` of type `u64`\n    fn new(value: NumberOrString, address: u64) -> Self { Self { value, address } }\n}\n",
                    "range": {
                      "end": {
                        "character": 0,
                        "line": 25
                      },
                      "start": {
                        "character": 0,
                        "line": 25
                      }
                    }
                  }
                ]
              }
            },
            "kind": "refactor",
            "title": "Generate `new`"
          }
        ]),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    code_action
}

pub(crate) async fn code_action_struct_with_type_params_request(
    service: &mut LspService<Backend>,
    uri: &Url,
) -> Request {
    let params = json!({
      "textDocument": {
        "uri": uri
      },
      "range": {
        "start": {
          "line": 4,
          "character": 9
        },
        "end": {
          "line": 4,
          "character": 9
        }
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
        json!([
          {
            "data": uri,
            "edit": {
              "changes": {
                uri_string.clone(): [
                  {
                    "newText": "\nimpl<T> Data<T> {\n    \n}\n",
                    "range": {
                      "end": {
                        "character": 0,
                        "line": 7
                      },
                      "start": {
                        "character": 0,
                        "line": 7
                      }
                    }
                  }
                ]
              }
            },
            "kind": "refactor",
            "title": "Generate impl for `Data`"
          },
          {
            "data": uri,
            "disabled": {
              "reason": "Struct Data already has a `new` function"
            },
            "edit": {
              "changes": {
                uri_string: [
                  {
                    "newText": "    fn new(value: T) -> Self { Self { value } }\n",
                    "range": {
                      "end": {
                        "character": 0,
                        "line": 9
                      },
                      "start": {
                        "character": 0,
                        "line": 9
                      }
                    }
                  }
                ]
              }
            },
            "kind": "refactor",
            "title": "Generate `new`"
          }
        ]),
    );
    assert_json_eq!(expected, response.ok().unwrap());
    code_action
}
