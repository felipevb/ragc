mod edit;
pub mod io;
pub mod periph;
mod ram;
mod regs;
mod rom;
mod special;
mod timer;

#[cfg(feature = "std")]
mod tests;

pub use io::AgcIoSpace;

use heapless::spsc::Producer;

//use log::{error, trace};

use self::periph::AgcIoPeriph;

use crate::consts;
use crate::consts::memmap;

// ============================================================================
// Trait Declarations
// ============================================================================
trait AgcMemType {
    fn read(&self, bank_idx: usize, bank_offset: usize) -> u16;
    fn write(&mut self, bank_idx: usize, bank_offset: usize, value: u16);
}

pub struct AgcMemoryMap<'a> {
    ram: ram::AgcRam,
    rom: rom::AgcRom<'a>,
    io: io::AgcIoSpace<'a>,
    edit: edit::AgcEditRegs,
    special: special::AgcSpecialRegs,
    timers: timer::AgcTimers,
    regs: regs::AgcRegs,
    rom_debug: bool,
    superbank: bool,
}

impl<'a> AgcMemoryMap<'a> {
    pub fn new_blank(rupt_tx: Producer<u8, 8>) -> AgcMemoryMap {
        AgcMemoryMap {
            #[cfg(feature = "std")]
            ram: ram::AgcRam::default(false),
            #[cfg(not(feature = "std"))]
            ram: ram::AgcRam::new(),
            rom: rom::AgcRom::blank(),
            io: io::AgcIoSpace::blank(),
            edit: edit::AgcEditRegs::new(),
            special: special::AgcSpecialRegs::new(rupt_tx),
            timers: timer::AgcTimers::new(),
            regs: regs::AgcRegs::new(),
            superbank: false,
            rom_debug: false,
        }
    }

    pub fn new(program: &'a [[u16; consts::ROM_BANK_NUM_WORDS]; consts::ROM_NUM_BANKS],
               downrupt: &'a mut dyn AgcIoPeriph,
               dsky: &'a mut dyn AgcIoPeriph,
               rupt_tx: Producer<u8, 8>) -> AgcMemoryMap<'a> {
        AgcMemoryMap {
            #[cfg(feature = "std")]
            ram: ram::AgcRam::default(false),
            #[cfg(not(feature = "std"))]
            ram: ram::AgcRam::new(),
            rom: rom::AgcRom::new(program),
            edit: edit::AgcEditRegs::new(),
            io: io::AgcIoSpace::new(downrupt, dsky),
            special: special::AgcSpecialRegs::new(rupt_tx),
            timers: timer::AgcTimers::new(),
            regs: regs::AgcRegs::new(),
            superbank: false,
            rom_debug: false,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.ram.reset();
        self.timers.reset();
        //self.io.reset();     // TODO: Implement a reset for IO Space
    }

    pub fn enable_rom_write(&mut self) {
        self.rom_debug = true;
    }

    pub fn fetch_timers(&mut self) -> &mut timer::AgcTimers {
        &mut self.timers
    }

    pub fn write_io(&mut self, idx: usize, value: u16) {
        match idx {
            consts::io::CHANNEL_L => {
                self.regs.write(0, consts::cpu::REG_L, value);
            }
            consts::io::CHANNEL_Q => {
                self.regs.write(0, consts::cpu::REG_Q, value);
            }
            consts::io::CHANNEL_SUPERBNK => {
                if value & 0x40 == 0x40 {
                    self.superbank = true;
                } else {
                    self.superbank = false;
                }
                self.io.write(idx, value);
            }
            consts::io::CHANNEL_CHAN13 => {
                match (value & 0o40000) == 0o40000 {
                    true => {
                        self.timers.set_time6_enable(true);
                    }
                    false => {
                        self.timers.set_time6_enable(false);
                    }
                }
                self.io.write(idx, value & 0o37777);
            }
            consts::io::CHANNEL_CHAN34 => {
                self.timers.set_downrupt_flags(1);
                self.io.write(idx, value);
            }
            consts::io::CHANNEL_CHAN35 => {
                self.timers.set_downrupt_flags(2);
                self.io.write(idx, value);
            }
            _ => {
                self.io.write(idx, value);
            }
        };
    }

    pub fn read_io(&mut self, idx: usize) -> u16 {
        match idx {
            consts::io::CHANNEL_L => self.regs.read(0, consts::cpu::REG_L),
            consts::io::CHANNEL_Q => self.regs.read(0, consts::cpu::REG_Q),
            consts::io::CHANNEL_HISCALAR => {
                let result = self.timers.read_scalar();
                ((result >> 14) & 0o37777) as u16
            }
            consts::io::CHANNEL_LOSCALAR => {
                let result = self.timers.read_scalar();
                (result & 0o37777) as u16
            }
            consts::io::CHANNEL_CHAN13 => {
                let mut res = self.io.read(idx);
                if self.timers.get_time6_enable() {
                    res |= 0o40000;
                }
                res
            }
            _ => self.io.read(idx),
        }
    }

    pub fn write(&mut self, idx: usize, val: u16) {
        //trace!("Write: 0x{:x}: 0o{:o}", idx, val);
        match idx {
            0o00..=0o17 => {
                self.regs.write(0, idx, val);
            }
            0o20..=0o23 => {
                self.edit.write(0, idx, val);
            }
            0o24..=0o31 => {
                self.timers.write(0, idx, val);
            }
            0o32..=0o60 => {
                self.special.write(0, idx, val);
            }
            memmap::AGC_MM_ERASABLE_START..=memmap::AGC_MM_ERASABLE_END => {
                if (idx >> 8) == 3 {
                    self.ram.write(self.regs.ebank, (idx & 0xff) as usize, val)
                } else {
                    self.ram.write(idx >> 8, (idx & 0xff) as usize, val)
                }
            }
            memmap::AGC_MM_FIXED_START..=memmap::AGC_MM_FIXED_END => {
                if self.rom_debug == false {
                    //error!("Writing to ROM location: {:x}", idx);
                    return;
                }

                let bank_idx = idx >> 10;
                if bank_idx == 1 {
                    self.rom.write(self.regs.fbank, (idx & 0x3ff) as usize, val)
                } else {
                    self.rom.write(bank_idx, (idx & 0x3ff) as usize, val)
                }
            }
            _ => {
                //error!("Unimplemented Memory Map Write (Addr: 0x{:x}", idx);
            }
        }
    }

    pub fn read(&self, idx: usize) -> u16 {
        let val = match idx {
            0o00..=0o17 => self.regs.read(0, (idx & 0xff) as usize),
            0o20..=0o23 => self.edit.read(0, idx),
            0o24..=0o31 => self.timers.read(0, idx),
            0o32..=0o60 => self.special.read(0, idx),
            memmap::AGC_MM_ERASABLE_START..=memmap::AGC_MM_ERASABLE_END => {
                if (idx >> 8) == 3 {
                    self.ram.read(self.regs.ebank, (idx & 0xff) as usize)
                } else {
                    self.ram.read(idx >> 8, (idx & 0xff) as usize)
                }
            }
            memmap::AGC_MM_FIXED_START..=memmap::AGC_MM_FIXED_END => {
                if (idx >> 10) == 1 {
                    //trace!("Reading from Windowed ROM: {:x} {:x}", self.regs.fbank, idx);
                    match self.regs.fbank {
                        0o30..=0o33 => {
                            if self.superbank == true {
                                self.rom
                                    .read(self.regs.fbank + 0o10, (idx & 0x3ff) as usize)
                            } else {
                                self.rom.read(self.regs.fbank, (idx & 0x3ff) as usize)
                            }
                        }
                        0o34..=0o37 => {
                            if self.superbank == true {
                                //error!("Inaccesible Bank with Superbank: {:x}", self.regs.fbank);
                                0
                            } else {
                                self.rom.read(self.regs.fbank, (idx & 0x3ff) as usize)
                            }
                        }
                        _ => self.rom.read(self.regs.fbank, (idx & 0x3ff) as usize),
                    }
                //self.rom.read(self.regs.fbank, (idx & 0x3ff) as usize)
                } else {
                    //trace!("Reading from Fixed ROM: {:x}", idx);
                    self.rom.read(idx >> 10, (idx & 0x3ff) as usize)
                }
            }
            _ => {
                //error!("Unimplemented Memory Map Read (Addr: 0x{:x}", idx);
                0
            }
        };

        //trace!("Read: 0x{:x}: 0o{:o}", idx, val);
        val
    }

    pub fn check_interrupts(&mut self) -> u16 {
        self.io.check_interrupt()
    }
}

/*
#[cfg(test)]
mod agc_memory_map_tests {
    use super::*;
    use crate::cpu;
    use heapless::Deque;

    ///
    /// Support function to initialize the ROM section of the AGU to a static
    /// value array. This is used to do initial testing with the ROM and RAM
    /// accessors
    ///
    fn init_static_rom() -> rom::AgcRom {
        let mut rom = rom::AgcRom::new();
        for bank_idx in 0..rom::ROM_BANKS_NUM {
            for bank_offset in 0..rom::ROM_BANK_NUM_WORDS {
                rom.write(bank_idx, bank_offset, (bank_idx + 1) as u16);
            }
        }
        rom
    }

    ///
    /// Support function to initialize the RAM section of the AGU to a static
    /// value array. This is used to do initial testing with the ROM and RAM
    /// accessors
    ///
    fn init_static_mem() -> AgcMemoryMap {
        let mut q1: heapless::spsc::Queue<u8, 8> = heapless::spsc::Queue::new();
        let (tx1, _rx1) = q1.split();

        let rom = init_static_rom();
        let mut mem = AgcMemoryMap::new_blank(tx1);
        mem.rom = rom;
        mem
    }

    #[test]
    ///
    /// This test is to perform a read test across the ROM section of the
    /// AGC memory map module. This test checks the specific two banks that are
    /// static within the AGC memory map layout. (0x800 through 0x1000). This is
    /// to ensure that those banks are bank 2 and bank 3 (starting at index 0)
    ///
    fn static_rom_read_test() {
        let mem = init_static_mem();
        for idx in 0x800..0x1000 {
            let bank_idx = (idx >> 10) as u16;
            assert_eq!(bank_idx + 1, mem.read(idx));
        }
    }

    #[test]
    ///
    /// This test is to perform a read test across the ROM section of the
    /// AGC memory map module. This test checks the window bank to ensure we can
    /// properly switch banks with the FB register. This test will only check
    /// the initial 32 banks available to ensure we can select to any of the ROM
    /// banks within the first 32 banks.
    ///
    fn windowed_rom_read_test() {
        let mut mem = init_static_mem();

        mem.write_io(7, 0);
        for bank_idx in 0..32 {
            for idx in 0x000..0x400 {
                let fb_val: u16 = bank_idx << 10;
                mem.write(cpu::REG_FB, 0);
                mem.write(cpu::REG_FB, fb_val);
                assert_eq!(
                    mem.read(0x400 + idx),
                    bank_idx + 1,
                    "Failed checking bank {:?}",
                    bank_idx
                );
            }
        }
    }

    #[test]
    ///
    /// This test is to perform a read test across the ROM section of the
    /// AGC memory map module. This test checks the window bank to ensure we can
    /// properly switch banks with the FB register. This test will check the
    /// functionality of the Superbank bit to ensure the appropriate range
    /// (bank 24 to 31) converts to the superbanks (32 to 40)
    ///
    fn windowed_rom_superbank_read_test() {
        let mut mem = init_static_mem();

        mem.write_io(7, 1 << 6);
        for bank_idx in 32..rom::ROM_BANKS_NUM {
            for idx in 0x000..0x400 {
                let fb_val: u16 = ((bank_idx - 8) << 10) as u16;
                mem.write(cpu::REG_FB, 0);
                mem.write(cpu::REG_FB, fb_val);

                let res = mem.read(0x400 + idx);
                let expect = (bank_idx + 1) as u16;
                assert_eq!(
                    res, expect,
                    "ROM Superbank Read: {:?} - {:?} vs {:?}",
                    bank_idx, res, expect
                );
            }
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// Testing BIT15 on Channel 0o13 which controls TIME6. The test demonstrates
    ///  - Starts with TIME6 disabled
    ///  - TIME6 is not decrementing during this time
    ///  - Write to TIME6 sets the bit and enables TIME6
    ///  - TIME6 decrements
    ///
    fn test_time6_enable_disable() {
        let mut q1: heapless::spsc::Queue<u8, 8> = heapless::spsc::Queue::new();
        let (tx, _rx) = q1.split();

        let mut mm = AgcMemoryMap::new_blank(tx);

        // Test to ensure the TIME6 is disabled via the IO Bit 15
        // By default, this should be disabled when first booted.
        assert_eq!(
            0o00000,
            mm.read_io(super::io::CHANNEL_CHAN13) & 0o40000,
            "TIME6 is enabled when it should be disabled by default"
        );

        // Put a default value into the timer to set a baseline to see if
        // TIME6 is truely disabled to start.
        mm.write(super::timer::MM_TIME6, 0o00007);
        assert_eq!(
            0o00007,
            mm.read(super::timer::MM_TIME6),
            "TIME6 is enabled when it should be disabled by default"
        );

        // Move enought MCTs to trigger a TIME6 DINC to occur. In this case,
        // there should not be movement in TIME6 or any DINCs
        let mut unprog = Deque::new();
        for _i in 0..55 {
            mm.timers.pump_mcts(1, &mut unprog);
        }
        assert_eq!(
            0o00000,
            mm.read_io(super::io::CHANNEL_CHAN13) & 0o40000,
            "TIME6 is enabled when it should be disabled by default"
        );
        assert_eq!(
            0o00007,
            mm.read(super::timer::MM_TIME6),
            "TIME6 value should not have changed"
        );
        assert_eq!(
            0,
            unprog.len(),
            "No unprog instructions should have appeared"
        );

        // Enable TIME6, validate the write worked, and timer works
        let val = mm.read_io(super::io::CHANNEL_CHAN13);
        mm.write_io(super::io::CHANNEL_CHAN13, 0o40000 | val);
        assert_eq!(
            0o40000,
            mm.read_io(super::io::CHANNEL_CHAN13) & 0o40000,
            "TIME6 is disabled when it should be enabled"
        );

        // Trigger a DINC and see the results
        for _i in 0..55 {
            mm.timers.pump_mcts(1, &mut unprog);
        }
        assert_eq!(
            0o00006,
            mm.read(super::timer::MM_TIME6),
            "TIME6 value should have decremented"
        );
        assert_eq!(1, unprog.len(), "Missing unprog seq");
    }

    #[test]
    ///
    /// # Description
    ///
    /// Testing BIT15 on Channel 0o13 which controls TIME6. The test demonstrates
    ///  - Bit is cleared when T6RUPT occurs
    ///
    fn test_time6_trupt_disable() {
        let mut q1: heapless::spsc::Queue<u8, 8> = heapless::spsc::Queue::new();
        let (tx, _rx) = q1.split();

        let mut mm = AgcMemoryMap::new_blank(tx);

        // Put a default value into the timer to set a baseline to see if
        // TIME6 is truely disabled to start.
        mm.write(super::timer::MM_TIME6, 0o00001);

        // Enable TIME6, validate the write worked, and timer works
        let val = mm.read_io(super::io::CHANNEL_CHAN13);
        mm.write_io(super::io::CHANNEL_CHAN13, 0o40000 | val);

        // Move enought MCTs to trigger a TIME6 DINC to occur. In this case,
        // there should not be movement in TIME6 or any DINCs
        let mut unprog = Deque::new();
        let mut interrupt_flags = 0;
        for _i in 0..200 {
            interrupt_flags |= mm.timers.pump_mcts(1, &mut unprog);
        }

        assert_eq!(1, unprog.len(), "Missing unprog seq");
        assert_eq!(
            0o0,
            mm.read(super::timer::MM_TIME6),
            "TIME6 value should be set to 0"
        );
        assert_eq!(
            1 << crate::cpu::RUPT_TIME6,
            interrupt_flags & (1 << crate::cpu::RUPT_TIME6),
            "Expecting TIME6 interrupt. Did not receive it"
        );
        assert_eq!(
            mm.read_io(super::io::CHANNEL_CHAN13 & 0o40000),
            0o0,
            "TIME6 should be disabled after a TRUPT"
        );
    }

    #[test]
    ///
    /// # Description
    ///
    fn test_io_reg_l_and_q() {
        let mut q1: heapless::spsc::Queue<u8, 8> = heapless::spsc::Queue::new();
        let (tx, _rx) = q1.split();

        let mut mm = AgcMemoryMap::new_blank(tx);
        for i in 0o000000..=0o177777 {
            mm.write(crate::cpu::REG_Q, i);
            mm.write(crate::cpu::REG_L, i);
            assert_eq!(i, mm.read_io(crate::mem::io::CHANNEL_Q), "Mismatch");
            assert_eq!(
                i & 0o77777,
                mm.read_io(crate::mem::io::CHANNEL_L),
                "Mismatch"
            );
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// Testing the HISCALAR and LOSCALAR work as intended
    ///
    fn test_scalar_registers() {
        let mut q1: heapless::spsc::Queue<u8, 8> = heapless::spsc::Queue::new();
        let (tx, _rx) = q1.split();

        let mut mm = AgcMemoryMap::new_blank(tx);
        let mut unprog = Deque::new();

        assert_eq!(0, mm.read_io(super::io::CHANNEL_HISCALAR), "Mismatch");
        assert_eq!(0, mm.read_io(super::io::CHANNEL_LOSCALAR), "Mismatch");

        // TOOD: Find out the right math on how this value makes it overflow the
        // way it does. It is "almost" there. It has something to do with the
        // `scalar_mct`s calculated.
        for _i in 0..27 {
            mm.timers.pump_mcts(1, &mut unprog);
        }

        assert_eq!(0, mm.read_io(super::io::CHANNEL_HISCALAR), "Mismatch");
        assert_eq!(1, mm.read_io(super::io::CHANNEL_LOSCALAR), "Mismatch");

        // TOOD: Find out the right math on how this value makes it overflow the
        // way it does. It is "almost" there. It has something to do with the
        // `scalar_mct`s calculated.
        for _i in 0..436883 {
            //436910
            mm.timers.pump_mcts(1, &mut unprog);
        }

        assert_eq!(1, mm.read_io(super::io::CHANNEL_HISCALAR), "Mismatch");
        assert_eq!(0, mm.read_io(super::io::CHANNEL_LOSCALAR), "Mismatch");
    }
}
*/