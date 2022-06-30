#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "heapless-periph")]
mod heapless;

#[cfg(feature = "heapless-periph")]
pub use crate::heapless::*;

#[cfg(feature = "vagc-periph")]
mod vagc;

#[cfg(feature = "vagc-periph")]
pub use vagc::*;

mod utils;
