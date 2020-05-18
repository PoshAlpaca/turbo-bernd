# TurboBernd

An HTTP server written in Rust

## Testing

### Coverage

```shell
docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin:0.12.2-nightly -o Html
```

### Load testing

```shell
cargo install drill
cargo run
drill --benchmark benchmark.yml
drill --benchmark sustained_load.yml
```

### Fuzz testing

```shell
cargo install afl
cd fuzzing
cargo afl build
cargo afl fuzz -i in/ -o out/ target/debug/fuzzing
```

- `in` is a directory containing input files that AFL uses as seeds. These files can have any name and their contents help AFL because it will not need to learn the correct text structure itself.
- `out` is a directory where AFL will store its state and results.

The output of AFL is explained [here](https://lcamtuf.coredump.cx/afl/status_screen.txt).
