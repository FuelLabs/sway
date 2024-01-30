use criterion::{black_box, criterion_group, Criterion};
use criterion::criterion_main;
use sway_core::Engines;

// Use Jemalloc during benchmarks
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn benchmarks(c: &mut Criterion) {
    let mut command = forc_doc::cli::Command::default();
    command.manifest_path = Some("../../sway-lib-std".to_string());
    command.silent = true;

    let engines = Engines::default();
    let tests_enabled = true;
    let mut compile_results = forc_pkg::check(
        &plan,
        BuildTarget::default(),
        build_instructions.silent,
        tests_enabled,
        &engines,
        None,
    )?;

    let manifest_file = forc_pkg::ManifestFile::from_dir(&command.manifest_path.as_ref().unwrap()).unwrap();

    let program_info = forc_doc::ProgramInfo {
        ty_program,
        engines: &engines,
        manifest: &manifest_file,
        pkg_manifest: pkg_manifest_file,
    };

    c.bench_function("test_forc_doc", |b| {
        b.iter(|| {
            forc_doc::build_docs(program_info, doc_path, &command).unwrap();
        })
    });
}

criterion_main!(benches);
criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10)).sample_size(10);
    targets = benchmarks
}

