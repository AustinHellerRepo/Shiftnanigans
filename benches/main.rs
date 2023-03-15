use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::segment_permutation_shifter::single_average::benches,
    benchmarks::pixel_board_randomizer::small_plus_sign::benches
}