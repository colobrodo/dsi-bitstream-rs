# dsi-bistream

A Rust implementation of read/write bit streams supporting several types
of instantaneous codes. It mimics the behavior of the analogous classes in
the [DSI Utilities](https://dsiutils.di.unimi.it/).

```rust
use dsi-bitstream::prelude::*;
// where the codes will be written to, this can also be a file, or a memory slice
let mut data = Vec::<u32>::new();
// write some data
{
    // create a codes writer
    let mut writer = BufferedBitStreamWrite::<BigEndian, _>::new(MemWordWriteVec::new(&mut data));
    // write 0 using 10 bits
    writer.write_bits(0, 10)?;
    // write 1 in unary
    writer.write_unary(1)?;
    // write 2 in gamma
    writer.write_gamma(2)?;
    // write 3 in delta
    writer.write_delta(3)?;
    // write 4 in zeta 3
    writer.write_zeta(3, 4)?;
}
// read them back
{
    // create a codes reader
    let mut reader = BufferedBitStreamRead::<BigEndian, u64, _>::new(MemWordRead::new(&data));
    assert_eq!(reader.read_bits(10)?, 0);
    assert_eq!(reader.write_unary()?, 1);
    assert_eq!(reader.write_gamma()?, 2);
    assert_eq!(reader.write_delta()?, 3);
    assert_eq!(reader.write_zeta(3)?, 4);
}
```

# Testing
```shell
cargo tarpaulin --engine llvm
```
If you also want to use the fuzzing cases use:
```shell
cargo tarpaulin --engine llvm --features="fuzz"
```
this will reproduce our selected corpus zip files at `tests/corpus/` and
run your local data corpus in `fuzz/corpus/`.
# Fuzzing
The fuzzing harnesses can be found in `dsi-bitstream::fuzz` so you can use 
whatever fuzzing framework you want. The simplest is `cargo-fuzz` which
can be installed as:
```shell
cargo install cargo-fuzz
```
To find the current targets:
```shell
cargo fuzz list
```
To start the fuzzing
```shell
cargo fuzz run codes
```
# Coverage
To compute the coverage in `lcov` format:
```shell
cargo tarpaulin --engine llvm --features="fuzz" -o lcov
```
# Corpus.zip
To update one of the selected corpus zip files the procedure is:
```shell
TARGET="codes"
# temp dir
mkdir tmp
# Extract the files
unzip "tests/corpus/${TARGET}.zip" -d tmp
# Merge and deduplicate the current corpus 
cargo fuzz run ${TARGET} -- -merge=1 tmp fuzz/corpus/${TARGET}
# Recomprss
zip tests/corpus/${TARGET}.zip tmp/*
# Delete tmp folder
rm -rfd tmp
```