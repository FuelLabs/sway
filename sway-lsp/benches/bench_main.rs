#![recursion_limit = "256"]

mod lsp_benchmarks;
use codspeed_criterion_compat::criterion_main;

// Use Jemalloc during benchmarks
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

criterion_main! {
    lsp_benchmarks::token_map::benches,
    lsp_benchmarks::requests::benches,
    lsp_benchmarks::compile::benches,
}
