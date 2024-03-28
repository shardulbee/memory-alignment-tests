use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[macro_use] extern crate byte_unit;

type Int = u64;


#[inline(never)]
#[no_mangle]
fn run_v4(vec: &mut [Int]) {
    for i in 0..vec.len() {
        let datum = &vec[i];
        unsafe {
            let _ = std::ptr::read_volatile(datum);
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let bytes_per_iteration = 8;
    let size_bytes = n_gib_bytes!(1);
    let size_in_elements = (size_bytes as u64 / bytes_per_iteration) as u64;
    let mut vec: Vec<Int> = Vec::new();
    for i in 0..size_in_elements {
        vec.push(i as Int)
    }
    let mut group = c.benchmark_group("run_v4");
    group.throughput(criterion::Throughput::Bytes(size_bytes as u64));
    group.bench_function("1 thread", |b| {
        // b.iter(|| println!("{}", memory_read_sequential_single_thread(&vec)))
        // b.iter(|| black_box(memory_read_sequential_single_thread_vectorized(&mut vec)))
        b.iter(|| black_box(run_v4(&mut vec)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
