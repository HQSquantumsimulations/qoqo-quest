use criterion::{criterion_group, Criterion};
use criterion::{BenchmarkId, Throughput};
use roqoqo::operations;
use roqoqo::prelude::EvaluatingBackend;
use roqoqo::Circuit;
use roqoqo_quest::Backend;

fn bench_run_long_circuit(c: &mut Criterion) {
    let mut subgroup = c.benchmark_group("run_simple_circuit");
    for number in [16].iter() {
        subgroup.throughput(Throughput::Bytes(*number as u64));
        subgroup.bench_with_input(
            BenchmarkId::from_parameter(number),
            number,
            |bench, &number| {
                bench.iter(|| {
                    let mut circuit = Circuit::new();
                    for j in 0..1000 {
                        for i in 0..number {
                            circuit += operations::RotateX::new(i, (0.01_f64 * j as f64).into());
                        }
                        for i in 0..number - 1 {
                            circuit += operations::CNOT::new(i, i + 1);
                        }
                    }
                    let backend = Backend::new(number);
                    let _res = backend.run_circuit(&circuit);
                });
            },
        );
    }
    subgroup.finish();
}

criterion_group!(benches, bench_run_long_circuit,);
