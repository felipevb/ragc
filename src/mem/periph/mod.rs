pub mod downrupt;
pub mod dsky;
pub mod engines;

pub trait Peripheral {
    fn is_interrupt(&mut self) -> u16;
}
