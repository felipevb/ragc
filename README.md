# ragc - The Rust Apollo Guidance Computer

`ragc` is an open source emulator, written in Rust, which implements the Block
II Apollo Guidance Computer (AGC), used for the Apollo lunar landing missions.
The goal for `ragc` is to provide a Rust implementation of the AGC which is able
to run real AGC software and provide mechanisms and hooks into `ragc` to simulate
sensor data to provide a way to simulate

## Building

Since `ragc` is written in the Rust language, the compilation process
is the same as any standard Cargo application / library. To compile `ragc` the
following commmand is used:

```rust
cargo build
```

As per the standard Cargo build flow, to compile the `ragc` as a release build,
the `--release` flag is needed. This flag instruct cargo to build the project as
a release build. The following command demonstrates this:

```rust
cargo build --release
```

## Running

The `ragc` CLI provides the ability to run AGC ROM code via file or through
prebuilt images provided by the `ragc-ropes` package. To run a prebuilt ROM
image, the following command demonstrates how to run the RETREAD50 program

```bash
cargo run -- retread50
```

The subcommand `file` can be used to run an external AGC ROM via a file. Example
binaries and ability to compile AGC4 code can be found at the VirtualAGC website.
The link to the VirtualAGC site be found in the [Resources](#Resources) section
of this document.

Once an AGC rope core binary is obtained, the following command demonstrates how
to run `ragc` with a custom ROM image:

```rust
cargo run -- file (binary path)
```

Addtional flags and options can be used while running `ragc`.
  - For the cargo build system, one can specify to use a release build to run
  with the `--release` flag. This is the same for `cargo run` as it is for
  `cargo build`.
  - The `ragc` leverages the standard Rust logging and uses an `env_logger`
  package to control the usage of the logging. To enable logging, one must
  specify `RUST_LOG` environment variable with the level of logging desired.
  For example:
    ```rust
    RUST_LOG=info cargo run --release file (binary path)
    ```
## Supporting Peripherials

`ragc` currently support integration with the following open source
peripherials to allow for interaction and simulation:
  - **`yaDSKY2` Support** - The yaDSKY2 project is an open source DSKY
implementation to allow for display and input. `ragc` by defaults connects to
the default port of the `yaDSKY2` application and handles proper indicator,
display, and keypress events.

# Resources

The following is a list of documentation and resources that were used to better
understand the Apollo Guidance computer architecture. This project wants to
highlight all the hard work that was performed in this list. Without this, the
RAGC project would have taken orders of magnitude long and would not be in existance
today without their contributions.

 - **[VirtualAGC Homepage](https://www.ibiblio.org/apollo)** - The VirtualAGC website
provides a wealth of documentation, videos, and knowledge for the Apollo Guidance
Computer (AGC). The site curates information about all the different Block versions
of the AGC and many components that were also involved in the overall Apollo space
mission. The VirtualAGC website also maintains many existing open source tools and
digitized Apollo software code to enable people to run and experiment with
an emulated AGC at home.

 - **[Michael Steil & Christian Hessmann - The Ultimate Apollo Guidance Computer
Talk](https://media.ccc.de/v/34c3-9064-the_ultimate_apollo_guidance_computer_talk)** -
In-depth and concise talk from 34C3 which went over the overall architecture, design,
implementation of the AGC computer and software. The following is a description
from the 34C3 website:

   - *The Apollo Guidance Computer was used onboard the Apollo spacecraft
   to support the Apollo moon landings between 1969 and 1972. This talk explains
   "everything about the AGC", including its quirky but clever hardware design,
   its revolutionary OS, and how its software allowed humans to reach and explore
   the moon.*

 - **[CuriousMarc YouTube Channel](http://youtube.com/curiousmarc)** - The
CuriousMarc YouTube channel documents vintage computer restoration, including the
Apollo Guidance Computer. The CuriousMarc channel contains a multi-part video
documenting the restoration project of the AGC for the 50th anniversity of the
Apollo moon landings.

 - **Frank O'Brien - The Apollo Guidance Computer Book: Architecture and Operations** -
The book can be found [here](https://www.springer.com/gp/book/9781441908766) for
reference. The following is a description about the book:

    - *By today's standards, the on-board computer used by the Apollo astronaut's
was a primitive affair, but in an age when most computers filled an entire room,
this was small, required little power, and incorporated several technologies that
were revolutionary for its time. This is the first book to fully describe the Apollo
guidance computer's architecture, Executive software, and the programs used by
astronauts. It describes the full range of technologies required in order to fly
the Apollo lunar missions, and whicn enabled the astronauts to fly to the Moon
and back!*

 - Various Apollo Guidance Computer digitized documentions found through the
VirtualAGC website and other various academic locations. These documents are
digial scans of the original memos, design documents, user guides, etc, from
MIT-IL / NASA between the late 1960s to late 1970s. Notable documents include:
   - ["Apollo Guidance, Navigation and Control Vol I of II - AGC4 Basic
   Training Manual" - MIT-IL](
       https://authors.library.caltech.edu/5456/1/hrst.mit.edu/hrs/apollo/public/archive/1704.pdf
    )

 - And the many other locations of informations that I may have lost track over
time. I hope to add more to this list over time as more resources are found.


# Wishlist / TODO

 - Fix bug where LUMINARY and COLOSSUS software does not print out AGC uptime (V16N65)
but is successful for RETREAD50.
 - Implement the IMU and inertial sensors in order to provide sensor data and emulate
one of the programs within LUMINARY/COLOSSUS
 - Reshape the codebase into a more library format. (i.e. - ragc-core, ragc-dsky, etc)
to reduce the amount of std libraries needed for the ragc-core
 - Revisit the DOWNRUPT and implement the interface to allow for integration
with any existing open source tools.
 - Many other bugs that I can't think of as of this writing.

# License

This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.