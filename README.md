# dhat-rs

**Warning:** *This crate is experimental. It relies on implementation techniques
that are hard to keep working for 100% of configurations. It may work fine for
you, or it may crash, hang, or otherwise do the wrong thing. Its maintenance is
not a high priority of the author. Support requests such as issues and pull
requests may receive slow responses, or no response at all. Sorry!*

This crate provides heap profiling and ad hoc profiling capabilities to Rust
programs, similar to those provided by [DHAT].

[DHAT]: https://www.valgrind.org/docs/manual/dh-manual.html

It also provides heap usage testing capabilities, which let you write tests
that check things like:
- "This code should do exactly 96 heap allocations".
- "The peak heap usage of this code should be less than 10 MiB".
- "This code should free all heap allocations before finishing".

It provides helpful details if these fail.

See the [crate documentation] for details on how to use it.

[crate documentation]: https://docs.rs/dhat

## License

Licensed under either of
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
