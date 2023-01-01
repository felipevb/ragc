### Overview

`ragc` is a top-level crate that implements the `ragc-core` emulator on a standard Rust environment on Linux and/or Windows. This crate implements a CLI interface to start and run a given rope core memory program.

`ragc` leverages the other crates within this repository to provide ease of use to start an emulated run, and provide a way to modularize the peripherals for reuse on other custom top-level implementations of `ragc`. The following describes the usages of the other crates within this repository:

 - `ragc-ropes` - This crate is a reference crate to store different implemented rope core ROMs that has been either flown on official Apollo missions, or other known ROM programs for either validation or debugging. These rope core ROMs were built with the VirtualAGC compilers (https://github.com/virtualagc/virtualagc) and the output binary is stored into the crate.
 - `ragc-periph` - The crate contains multiple implementations of the standad peripherals of the AGC computer (DSKY, DOWNRUPT, etc). The `ragc` cli program leverages the implementation of the `yaagc` protocol (https://www.ibiblio.org/apollo/developer.html#sendrecv_Protocol) on a `std` Rust environment (Linux/Windows).


### Build

Since `ragc` is written in the Rust language, the compilation process is the same as any standard Cargo application / library. To compile `ragc` the following commmand is used:

```rust
cd ./ragc
cargo build
```

As per the standard Cargo build flow, to compile the `ragc` as a release build, the `--release` flag is needed. This flag instruct cargo to build the project as a release build. The following command demonstrates this:

```rust
cd ./ragc
cargo build --release
```

## Usage

The `ragc` CLI provides the ability to run AGC ROM code via file or through prebuilt images provided by the `ragc-ropes` package. To run a prebuilt ROM image, the following command demonstrates how to run the RETREAD50 program

```bash
cd ./ragc
cargo run -- retread50
```

The subcommand `file` can be used to run an external AGC ROM via a file. Example binaries and ability to compile AGC4 code can be found at the VirtualAGC website. The link to the VirtualAGC site be found in the [Resources](#Resources) section of this document.

Once an AGC rope core binary is obtained, the following command demonstrates how to run `ragc` with a custom ROM image:

```rust
cargo run -- file (binary path)
```
