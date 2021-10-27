use criterion::criterion_main;

mod benchmark_run_circuit;

criterion_main! {
    benchmark_run_circuit::benches,
}
