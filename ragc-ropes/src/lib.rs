#![no_std]

const ROM_BANKS_NUM: usize = 36;
const ROM_BANK_NUM_WORDS: usize = 1024;

macro_rules! include_transmute {
    ($file:expr) => {
        &core::mem::transmute(*include_bytes!($file))
    };
}
pub static RETREAD50_ROPE: &'static [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM] = unsafe { include_transmute!("../res/RETREAD50.ROM") };
pub static VALIDATION_ROPE: &'static [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM] = unsafe { include_transmute!("../res/VALIDATION.ROM") };
pub static LUMINARY131_ROPE: &'static [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM] = unsafe { include_transmute!("../res/LUMINARY131.ROM") };
pub static BLANK_ROPE: &'static [[u16; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM] = &[[0; ROM_BANK_NUM_WORDS]; ROM_BANKS_NUM];
