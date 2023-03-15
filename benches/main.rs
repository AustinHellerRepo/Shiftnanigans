use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::segment_permutation_shifter_benchmark::benches
}