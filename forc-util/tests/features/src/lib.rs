#![cfg(test)]

#[test]
fn common_helpers() {
    assert_eq!(forc_util::kebab_to_snake_case("my-project"), "my_project");
    assert_eq!(
        forc_util::default_output_directory(std::path::Path::new("project")),
        std::path::Path::new("project/out")
    );
    fn fail() -> forc_util::ForcResult<()> {
        forc_util::forc_result_bail!("example error");
    }
    assert_eq!(fail().unwrap_err().to_string(), "example error");
}

#[cfg(any(feature = "defaults", feature = "restricted"))]
#[test]
fn names_and_regex() {
    assert!(forc_util::validate_project_name("my-project").is_ok());
    assert!(forc_util::validate_name("fn", "project name").is_err());
    assert!(forc_util::restricted::is_keyword("fn"));
    assert!(forc_util::Regex::new("project")
        .unwrap()
        .is_match("my-project"));
}

#[cfg(any(feature = "defaults", feature = "bytecode"))]
#[test]
fn bytecode_api() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/bytecode/debug-counter.bin");
    assert_eq!(
        forc_util::bytecode::get_bytecode_id(&fixture).unwrap(),
        "e65aa988cae1041b64dc2d85e496eed0e8a1d8105133bd313c17645a1859d53b"
    );
    assert!(forc_util::bytecode::parse_bytecode_to_instructions(fixture)
        .unwrap()
        .next()
        .is_some());
}

#[cfg(any(feature = "defaults", feature = "fs-locking"))]
#[test]
fn locking_api() {
    let mut lock = forc_util::path_lock("feature-consumer").unwrap();
    let _guard = lock.write().unwrap();
    assert!(!forc_util::fs_locking::is_file_dirty("feature-consumer"));
}

#[cfg(any(feature = "defaults", feature = "diagnostics"))]
#[test]
fn diagnostics_api() {
    let _renderer = forc_util::create_diagnostics_renderer();
    let _ = forc_util::program_type_str;
    let _ = forc_util::print_compiling;
    let _ = forc_util::print_infos;
    let _ = forc_util::print_warnings;
    let _ = forc_util::print_on_failure;
    let _ = forc_util::format_diagnostic;
}

#[cfg(feature = "tx")]
#[test]
fn transaction_api() {
    assert_eq!(
        forc_util::tx_utils::format_log_receipts(&[], false).unwrap(),
        "[]"
    );
    assert!(forc_util::tx_utils::Salt::default().salt.is_none());
    let _ = forc_util::tx_utils::decode_log_data;
    let _ = forc_util::tx_utils::decode_fuel_vm_log_data;
    let _ = forc_util::tx_utils::revert_info_from_receipts;
}

#[cfg(any(feature = "defaults", feature = "cli"))]
mod cli {
    #[derive(clap::Parser)]
    struct Args {}

    forc_util::cli_examples! {
        crate::cli::Args {
            [ Empty Arguments => "example" ]
        }
    }

    #[test]
    fn cli_result_and_help() {
        let result: forc_util::ForcCliResult<()> = Ok(()).into();
        assert_eq!(
            std::process::Termination::report(result),
            std::process::ExitCode::SUCCESS
        );
        assert!(help().contains("EXAMPLES:"));
        assert!(examples().contains("example"));
    }

    mod block_form {
        forc_util::cli_examples! {
            { |args: Vec<String>| -> Result<(), ()> {
                if args == ["example", "hello world"] { Ok(()) } else { Err(()) }
            } } {
                [ Quoted Arguments => "example 'hello world'" ]
            }
        }

        #[test]
        fn block_help() {
            assert!(help().contains("EXAMPLES:"));
        }
    }
}
