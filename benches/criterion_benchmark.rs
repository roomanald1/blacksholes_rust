// benches/criterion_benchmark.rs
use criterion::{
    Criterion,
    black_box,
    criterion_group,
    criterion_main
};
use pprof::criterion::{PProfProfiler, Output};
fn criterion_benchmark(c: &mut Criterion) {

    // Force symbol preservation
    #[inline(never)]
    #[unsafe(export_name = "profiled_blackscholes")]  // This prevents name mangling
    pub fn profiled_blackscholes_call() -> f64 {
        blacksholes_rust::blacksholes::calc_call(
            black_box(100.0),
            black_box(100.0),
            black_box(1.0),
            black_box(0.05),
            black_box(0.2)
        )
    }
    c.bench_function("calc_call", |b| {
        b.iter(|| {
            unsafe {
                black_box(profiled_blackscholes_call())
            }
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
   targets = criterion_benchmark
}
criterion_main!(benches);