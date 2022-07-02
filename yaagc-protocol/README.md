# yaagc-protocol

`yaagc-protocol` is a Rust Implementation of YAAGC Socket Protocol from the
VirtualAGC project.

## Building

Since `yaagc-protocol` is written in the Rust language, the compilation process
is the same as any standard Cargo application / library. To compile
`yaagc-protocol` the following commmand is used:

```rust
cargo build
```
# Wishlist / TODO

 - Create a structure based implementation to provide a `serialize` and
 `deserialize` function for easier parsing and manipulation of the data.
 - Implement the `t` flag which provides a hard change to a counter address
 - Implement the `u` flag which provides alternative modes of the IO channel. An
 example of this is IO channels that do one function for the Command Module and
 another function for the Lunal Lander Module.
 - Add examples on how to build, create a Socket and send data over.
 - Add unittests to provide initial testing of the library crate.

# License

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.