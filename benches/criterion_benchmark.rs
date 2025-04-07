// benches/criterion_benchmark.rs
use criterion::{
    Criterion,
    black_box,
    criterion_group,
    criterion_main
};
use pprof::criterion::{PProfProfiler, Output};
use blacksholes_rust::monte_carlo::monte_carlo;
use blacksholes_rust::option_pricer::{OptionPricer, OptionValue};
use blacksholes_rust::utils::OptionType;

fn criterion_benchmark(c: &mut Criterion) {

    // Force symbol preservation
    #[inline(never)]
    #[unsafe(export_name = "profiled_blackscholes")]  // This prevents name mangling
    pub fn profiled_blackscholes_call() -> OptionValue {
        let mut option_pricer = black_box(OptionPricer::new(black_box(100.0), black_box(100.0), black_box(1.0), black_box(0.05), black_box(0.2)));
        option_pricer.calculate_option(black_box(OptionType::Call))
    }
    c.bench_function("calc_call", |b| {
        b.iter(|| {
            unsafe {
                black_box(profiled_blackscholes_call())
            }
        });
    });



    // Force symbol preservation
    #[inline(never)]
    #[unsafe(export_name = "profiled_monte_carlo")]  // This prevents name mangling
    pub fn profiled_monte_carlo() -> f64 {
        monte_carlo(black_box(OptionType::Call), black_box(100.0), black_box(100.0), black_box(1.0), black_box(0.05), black_box(0.2), black_box(1_000_000))
    }
    c.bench_function("calc_call_monte_carlo", |b| {
        b.iter(|| {
            unsafe {
                black_box(profiled_monte_carlo())
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