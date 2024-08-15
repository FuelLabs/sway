use e2e_tests::*;
use e2e_vm_tests::{filter_tests, TestContext};
use forc_tracing::init_tracing_subscriber;
use libtest_mimic::{Arguments, Trial};

pub fn main() {
    init_tracing_subscriber(Default::default());

    let mut args = Arguments::from_args();
    args.nocapture = true;
    args.test_threads = Some(1);

    e2e_tests::test_consistency::check().unwrap();
    e2e_tests::reduced_std_libs::create().unwrap();

    let (filter_config, run_config) = e2e_tests::configs_from_cli(&Cli {
        include: None,
        exclude: None,
        skip_until: None,
        abi_only: false,
        exclude_core: false,
        exclude_std: false,
        contract_only: false,
        first_only: false,
        verbose: false,
        release: false,
        locked: false,
        build_target: Some("fuel".into()),
        no_encoding_v1: false,
        update_output_files: false,
        print_ir: None,
        print_asm: None,
        print_bytecode: false,
        snapshot_only: true,
    });

    let mut tests = e2e_tests::e2e_vm_tests::discover_test_configs(&run_config).unwrap();
    filter_tests(&filter_config, &mut tests);
    let tests = tests
        .into_iter()
        .map(|test| {
            let run_config = run_config.clone();
            let filter_config = filter_config.clone();
            Trial::test(test.name.clone(), move || {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let context = TestContext {
                        deployed_contracts: Default::default(),
                    };
                    let mut failed_tests = vec![];
                    let mut qty_failed = 0;
                    e2e_vm_tests::run_test(
                        &context,
                        &run_config,
                        &filter_config,
                        0,
                        test,
                        &mut failed_tests,
                        &mut qty_failed,
                        true,
                    )
                    .await;
                });
                Ok(())
            })
        })
        .collect();

    libtest_mimic::run(&args, tests).exit();
}
