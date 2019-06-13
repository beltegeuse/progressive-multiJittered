# Progressive Multi-Jittered sequences
A simple Implementations of "Progressive Multi-Jittered Sample Sequences", EGSR 2018 in Rust.

These are the performance reported for 1024 samples:
```
$ cargo bench
running 4 tests
test bench_1024_jitter       ... bench:      30,729 ns/iter (+/- 192)
test bench_1024_mulijitter   ... bench:     281,247 ns/iter (+/- 5,780)
test bench_1024_mulijitter02 ... bench:  29,867,256 ns/iter (+/- 1,550,394)
test bench_1024_random       ... bench:      18,374 ns/iter (+/- 291)
```

You can also use `cargo run --release --` to visualize the distributions.

## TODO
- Implement Matt Pharr's technique "Efficient Generation of Points that Satisfy Two-Dimensional Elementary Intervals"
- Make the library more usable for a rendering system