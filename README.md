# `dhat`

This crate provides heap profiling and ad hoc profiling capabilities to Rust
programs, similar to those provided by [DHAT].

[DHAT]: https://www.valgrind.org/docs/manual/dh-manual.html

## A minimal heap profiling example

When the following program terminates, it will print a `dhat-heap.json` file
that can be viewed with DHAT's viewer.
```
use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

fn main() {
    let _dhat = Dhat::start_heap_profiling();
    println!("Hello, world!");
}
```

The crate's documentation has more details.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
