mod lsp_benchmarks;
use criterion::criterion_main;

criterion_main! {
    // lsp_benchmarks::token_map::benches,
    lsp_benchmarks::requests::benches,
}
