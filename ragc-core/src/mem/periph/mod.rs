pub mod downrupt;
pub mod dsky;
pub mod engines;

pub trait Peripheral {
    fn is_interrupt(&mut self) -> u16;
}
pub trait AgcIoPeriph {
    fn read(&self, _channel_idx: usize) -> u16;
    fn write(&mut self, channel_idx: usize, value: u16);
    fn is_interrupt(&mut self) -> u16;
}