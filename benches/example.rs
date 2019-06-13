#[macro_use]
extern crate bencher;
use bencher::Bencher;

extern crate seq;
use seq::*;

fn bench_1024_random(bench: &mut Bencher) {
    let n = 1024;
    bench.iter(|| {
        let mut gen = Uniform::default();
        for _ in 0..n {
            gen.generate();
        }
    });
}

fn bench_1024_jitter(bench: &mut Bencher) {
    let n = 1024;
    bench.iter(|| {
        let mut gen = Jittered::default();
        for _ in 0..n {
            gen.generate();
        }
    });
}

fn bench_1024_mulijitter(bench: &mut Bencher) {
    let n = 1024;
    bench.iter(|| {
        let mut gen = MultiJittered::<HVElementaryElement>::default();
        for _ in 0..n {
            gen.generate();
        }
    });
}

fn bench_1024_mulijitter02(bench: &mut Bencher) {
    let n = 1024;
    bench.iter(|| {
        let mut gen = MultiJittered::<ElementaryElement02>::default();
        for _ in 0..n {
            gen.generate();
        }
    });
}

benchmark_group!(
    benches,
    bench_1024_random,
    bench_1024_jitter,
    bench_1024_mulijitter,
    bench_1024_mulijitter02
);
benchmark_main!(benches);
