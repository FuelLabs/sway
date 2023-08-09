mod benchmarks;
use criterion::criterion_main;

criterion_main! {
    benchmarks::token_map::benches,
    benchmarks::requests::benches,
}
