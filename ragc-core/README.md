The `ragc-core` crate is a `nostd` capable Apollo Guidance Computer (AGC4) emulator. The goal for the crate is to maintain portability across multiple platforms, either standard Operating Systems with `std` support (Window, Linux, etc) and Embedded IoT platforms (RP2040, etc).

As of this writing (01/2023), the packages only leverages `log` and `heapless` crates to maintain minimum dependencies and to maintain a `nostd` capability.

## Peripheral and IO Modularity

Throughout the development of `ragc` and `ragc-core`, the implementation of how the emulation engine interacts with peripherals and general I/O space has constantly evolved. As time progressed, the support for `nostd` and portability to an embedded environment has pushed the design of the I/O space to support more a flexible and modular design. To date, there is basic support of this, with needs for improvement.

The goal is for some top-level crate to create the platform dependent modules needed to run, pass those into the `ragc-core` engine. The following code snippet is a simple example of this for a standard Rust environment on Windows or Linux:

```rust
    // Create a DSKY peripheral module which will handle all the I/O interactions
    // which relates to the DSKY (Display and Keyboard) within the AGC4.
    let mut dsky = ragc_periph::dsky::DskyDisplay::new();

    // Downrupt is an internal peripheral which sends two 15 Bit words every 20ms.
    // An interrupt would occur every 20ms, which the AGC code would push
    // telemetry data to the ground station. It is envisioned that there will be
    // multiple implementations of the Downrupt peripheral to capture and process
    // the data being sent out.
    let mut downrupt = ragc_periph::downrupt::DownruptPeriph::new();

    // Pass the two peripheral modules to construct the general AGC memory map.
    // The memory map interface contains I/O space registers at specific offsets
    // in the address space.
    let mm = mem::AgcMemoryMap::new(&rope, &mut downrupt, &mut dsky, ....);

    // Pass the memory space into the AGC CPU emulator to use. This is the main
    // interface in which data and I/O is interacted.
    let mut cpu = cpu::AgcCpu::new(mm);

    // Reset and take a step forward to the next instruction
    // within the AGC space.
    cpu.reset();
    loop {
        cpu.step();
    }
```

A working version can be found in the other crates within the repository (`ragc` for Rust `std` Linux/Windows, `ragc-targets` for more embedded examples).

## Feature Flags

The following is a list of flags that `ragc-core` supports:

 - `std` - This flag is used to enable the use of Rust's `std` crate and default features. Currently, the use of `std` is used in the following portions of the code:
    - **Module Tests** - To enable unit-testing on each of the instructions and implemented modules, the use of `std` is needed.
    - **RAM Save State** - The AGC4 computer implemented RAM with the use of magnetic core rings. The artifact of this allows the computer to restart with the existing data when it last powered down. To implement this feature, `ragc-core` saves the RAM contents periodically to file. When the `ragc-core` starts up, it can load the RAM state file and resume execution from there.
