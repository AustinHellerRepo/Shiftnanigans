use std::rc::Rc;

use austinhellerrepo_common_rust::shifter::{segment_permutation_shifter::{SegmentPermutationShifter, Segment}, Shifter};
use criterion::{black_box, criterion_group, Criterion};

fn single_shifter(bounding_length: usize) {
    let mut shifter = SegmentPermutationShifter::new(
        vec![
            Rc::new(Segment::new(1))
        ],
        (10, 100),
        bounding_length,
        true,
        1,
        false
    );
    shifter.randomize();
    shifter.try_forward();
    for _ in 0..bounding_length {
        shifter.try_increment();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("single_shifter: 41", |b| b.iter(|| single_shifter(black_box(41))));
}

criterion_group!(benches, criterion_benchmark);