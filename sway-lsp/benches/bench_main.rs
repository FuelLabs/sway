mod lsp_benchmarks;
use criterion::criterion_main;

// Use Jemalloc during benchmarks
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

criterion_main! {
    lsp_benchmarks::token_map::benches,
    lsp_benchmarks::requests::benches,
    lsp_benchmarks::compile::benches,
}
