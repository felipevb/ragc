#![no_std]

mod utils;

#[cfg(feature = "heapless-periph")]
mod heapless;

#[cfg(feature = "heapless-periph")]
pub use crate::heapless::*;

#[cfg(feature = "vagc_periph")]
mod vagc;

#[cfg(feature = "vagc_periph")]
pub use vagc::*;

