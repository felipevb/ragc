use log::{debug, info, trace, warn};

use crossbeam_channel::Sender;

use crate::disasm::disasm;
use crate::instr::{AgcArith, AgcControlFlow, AgcInterrupt, AgcIo, AgcLoadStore, AgcLogic};
use crate::instr::{AgcInst, AgcMnem};
use crate::mem::AgcMemoryMap;
use crate::utils::{overflow_correction, s15_add, sign_extend};

pub const REG_A: usize = 0x0;
pub const REG_L: usize = 0x1; // Original Name
pub const _REG_B: usize = 0x1;
pub const REG_Q: usize = 0x02; // Original Name
pub const REG_LR: usize = 0x2;
pub const REG_EB: usize = 0x3;
pub const REG_FB: usize = 0x4;
pub const REG_Z: usize = 0x05;
pub const REG_PC: usize = 0x05;
pub const REG_BB: usize = 0x6;
pub const REG_ZERO: usize = 0x7;
pub const _REG_A_SHADOW: usize = 0x8;
pub const _REG_B_SHADOW: usize = 0x9;
pub const _REG_LR_SHADOW: usize = 0xA;
pub const _REG_EB_SHADOW: usize = 0xB;
pub const _REG_FB_SHADOW: usize = 0xC;
pub const REG_PC_SHADOW: usize = 0xD;
pub const _REG_BB_SHADOW: usize = 0xE;

pub const REG_IR: usize = 0xF;
pub const _REG_MAX: usize = 0x10;

pub const _RUPT_RESET: u8 = 0x0;
pub const RUPT_TIME6: u8 = 0x1;
pub const RUPT_TIME5: u8 = 0x2;
pub const RUPT_TIME3: u8 = 0x3;
pub const RUPT_TIME4: u8 = 0x4;
pub const RUPT_KEY1: u8 = 0x5;
pub const _RUPT_KEY2: u8 = 0x6;
pub const _RUPT_UPRUPT: u8 = 0x7;
pub const RUPT_DOWNRUPT: u8 = 0x8;
pub const _RUPT_RADAR: u8 = 0x9;
pub const _RUPT_HANDRUPT: u8 = 0xA;

pub const NIGHTWATCH_TIME: u32 = 1920000000 / 11700;

// Each TC/TCF is 1 cycle, so we just need to have to know how many cycles it
// takes for 15ms and thats how many TC/TCF instructions we have to see in
// sequence to reset.
pub const TCMONITOR_COUNT: u32 = 15000000 / 11700;

pub const RUPT_LOCK_COUNT: i32 = 300000000 / 11700;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AgcUnprogSeq {
    PINC,
    PCDU,
    MINC,
    MCDU,
    DINC,
    SHINC,
    SHANC,
    INOTRD,
    INOTLD,
    FETCH,
    STORE,
    GOJ,
    TCSAJ,
    RUPT,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AgcOverflow {
    None,
    Positive,
    Negative,
}

trait AgcUnprogInstr {
    fn handle_goj(&mut self) -> u16;
}

#[allow(dead_code)]
pub struct AgcCpu {
    mem: AgcMemoryMap,
    pub ir: u16,
    pub idx_val: u16,
    pub ec_flag: bool,
    //pub v: AgcOverflow,
    pub total_cycles: usize,
    pub cycles: u16,
    mct_counter: f64,
    timer_counter: u8,

    pub gint: bool,
    pub is_irupt: bool,

    unprog: std::collections::VecDeque<AgcUnprogSeq>,
    pub rupt: u16,
    incr_tx: Sender<()>,

    nightwatch: u16,
    nightwatch_cycles: u32,

    tc_count: u32,
    non_tc_count: u32,

    ruptlock_count: i32,
}

impl AgcUnprogInstr for AgcCpu {
    fn handle_goj(&mut self) -> u16 {
        debug!("Handling GOJ (Restart of AGC)");

        // Within Memo #340, the following is listed on what GOJAM actions should
        // be performed.
        self.write_io(5, 0); // PYJETS
        self.write_io(6, 0); // ROLLJETS
        self.write_io(10, 0); // DSKY
        self.write_io(11, 0); // DSALMOUT
        self.write_io(12, 0); // 12
        self.write_io(13, 0); // 13
        self.write_io(13, 0); // 14
        self.write_io(34, 0); // DOWNWORD1
        self.write_io(34, 0); // DOWNWORD2

        // Clearing Bit 11 of Channel 33
        let val = self.read_io(33);
        self.write_io(33, val & 0o75777);

        // Reset a bunch of hardware logic. This section here is to accomodate
        // any future implementations of reseting any of this hardware logic.
        // One possible reset is all rupt requests.
        self.gint = false;
        self.is_irupt = false;

        // TODO: Check RUPT requests
        // TODO: Check how to clear TIMER requests
        // TODO: Check how to clear UPRUPT requests

        // Generate Restart light and a bunch of other stuff mentioned in the
        // memo. Again, this section is to handle the generation of signals.
        // TODO: Restart Light Generation.

        // Internal RAGC stuff:
        // Reset the NIGHT WATCHMAN, TC TRAP monitors
        self.tc_count = 0;
        self.non_tc_count = 0;

        // Reset the CPU by resetting to address 0x800
        self.restart();

        2
    }
}

impl AgcCpu {
    ///
    /// ## `calculate_instr_data` Function
    ///
    /// The following function provides a common code function to get the
    /// current instruction data based on the current INDEX value and the
    /// precalculated IR data.
    ///
    fn calculate_instr_data(&self) -> u16 {
        //let mut inst_data = (self.ir + s15_ones_to_twos(self.idx_val)) & 0x7FFF;
        let mut inst_data = s15_add(self.ir, self.idx_val);
        if self.ec_flag {
            inst_data = inst_data | 0x8000;
        }
        inst_data
    }

    pub fn new(memmap: AgcMemoryMap, incr_tx: Sender<()>) -> AgcCpu {
        let mut cpu = AgcCpu {
            mem: memmap,
            ir: 0x0,
            ec_flag: false,
            idx_val: 0x0,
            incr_tx: incr_tx,

            unprog: std::collections::VecDeque::new(),

            total_cycles: 0,
            cycles: 0,
            mct_counter: 0.0,
            timer_counter: 0,

            gint: false,
            is_irupt: false,
            rupt: 1 << RUPT_DOWNRUPT,

            nightwatch: 0,
            nightwatch_cycles: 0,
            tc_count: 0,
            non_tc_count: 0,
            ruptlock_count: 0,
        };

        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        // Initial PC value for the AGC CPU is at 0x800
        // so set the PC value to that.
        self.update_pc(0x800);
        self.gint = false;
    }

    fn restart(&mut self) {
        // Initial PC value for the AGC CPU is at 0x800
        // so set the PC value to that.
        self.update_pc(0x800);
        self.gint = false;

        // Since it's a restart, light up the DSKY to indicate a restart
        let io_val = self.read_io(0o163);
        self.write_io(0o163, 0o200 | io_val);
    }

    pub fn update_pc(&mut self, val: u16) {
        self.write(REG_PC, val);
        self.ir = self.read(val as usize);
    }

    pub fn set_unprog_seq(&mut self, unprog_type: AgcUnprogSeq) {
        debug!("Setting UnprogSeq: {:?}", unprog_type);
        self.unprog.push_back(unprog_type);
    }

    pub fn check_editing(&mut self, k: usize) {
        match k {
            0o20 | 0o21 | 0o22 | 0o23 => {
                let val = self.read_s15(k);
                self.write_s15(k, val);
            }
            _ => {}
        }
    }

    pub fn read(&mut self, idx: usize) -> u16 {
        if idx == 0o067 {
            self.nightwatch += 1;
        }
        self.mem.read(idx)
    }

    ///
    ///  ## `read_s16` Function
    ///
    ///  Reads a sign value from the memory interface. This function
    ///   performs sign-extends the 15-bit or 16-bit value into a
    ///   16-bit value
    ///
    ///  ## Arguments
    ///   - `idx` - Address into the memory space to read from
    ///
    ///  ## Result
    ///  Returns a u16 that is sign extended to 16 bits
    ///
    ///
    pub fn read_s16(&mut self, idx: usize) -> u16 {
        match idx {
            // Handle the case where the source of memory will
            // already return 16-bits. Do not perform a sign extend
            // of the value.
            REG_A | REG_Q => self.read(idx),

            // Otherwise, sign extend.
            _ => sign_extend(self.read(idx)),
        }
    }

    ///
    ///  ## `read_s15` Function
    ///
    ///  ## Arguments
    ///   - `idx` - Address into the memory space to read from
    ///
    ///  ## Result
    ///  Returns a u16 that is a signed 15 bits
    ///
    pub fn read_s15(&mut self, idx: usize) -> u16 {
        match idx {
            // With this, we are reading from a S16 bit value. We need to overflow
            // correct to a s15 bit value.
            REG_A | REG_Q => overflow_correction(self.read(idx)) & 0x7FFF,

            // Otherwise, just read the value
            _ => self.read(idx) & 0x7FFF,
        }
    }

    ///
    ///  Perform a write to a memory interface with a signed 16-bit value. For
    ///  registers A and Q, this is just a simple write with all 16 bits.
    ///  Otherwise, perform an overflow corrections to bring it to 15 bits
    ///
    /// # Arguments
    ///   - `idx` - Address into the memory space to write to
    ///   - `value` - Value to write that needs to be overflow corrected.
    ///               It is assumed to be a 16-bit signed value
    ///
    pub fn write_s16(&mut self, idx: usize, value: u16) {
        match idx {
            // Handle the case where the source of memory will
            // already return 16-bits. No need to change bit width
            REG_A | REG_Q => {
                self.write(idx, value);
            }

            // Otherwise, correct the overflow, since we have 16 bits and
            // we need to reduce to 15 bits. This assumes an overflow
            // correction for the 16bit to 15bit transfer.
            _ => {
                self.write(idx, overflow_correction(value) & 0o77777);
            }
        };
    }

    ///
    /// Perform a write to a memory interface with a signed 15-bit value. For
    ///  registers A and Q, the value must be sign extended to be 16-bits.
    /// Otherwise, the value is written as is.
    ///
    /// # Arguments
    ///   - `idx` - Address into the memory space to write to
    ///   - `value` - Value to write that needs to be overflow corrected.
    ///               It is assumed to be a 15-bit signed value
    ///
    pub fn write_s15(&mut self, idx: usize, value: u16) {
        match idx {
            // We are trying to write a signed 15-bit value to a 16-bit register
            // so we need to sign extend the value into the registers
            REG_A | REG_Q => {
                self.write(idx, sign_extend(value));
            }

            // Otherwise, just write the value. Mask it to ensure that the value
            // for Bit15 is not set, just in case.
            _ => {
                self.write(idx, value & 0o77777);
            }
        };
    }

    pub fn write(&mut self, idx: usize, val: u16) {
        if idx == 0o067 {
            self.nightwatch += 1;
        }
        self.mem.write(idx, val)
    }

    #[allow(dead_code)]
    pub fn read_dp(&mut self, idx: usize) -> u32 {
        let upper: u32 = self.read_s15(idx) as u32;
        let lower: u32 = self.read_s15(idx + 1) as u32;

        match (upper & 0o40000) == (lower & 0o40000) {
            true => (upper << 14) | (lower & 0o37777),
            false => {
                let mut res = if lower & 0o40000 == 0o40000 {
                    let mut val: u32 = upper << 14;
                    val += lower | 0o3777740000;
                    val
                } else {
                    let mut val: u32 = (upper + 1) << 14;
                    val += lower - 1;
                    val
                };

                if res & 0o4000000000 == 0o4000000000 {
                    res += 1;
                }
                res & 0o3777777777
            }
        }
    }

    ///
    /// ## `write_dp` Function
    ///
    /// Perform a double precision write to a memory interface with a signed
    /// 28-bit value.
    ///
    /// ### Arguments
    ///   - `idx` - Address into the memory space to write to
    ///   - `value` - Value to write that needs to be overflow corrected.
    ///               It is assumed to be a 28-bit signed value
    ///
    pub fn write_dp(&mut self, idx: usize, val: u32) {
        let upper = ((val >> 14) & 0o77777) as u16;
        let lower = (val & 0o37777) as u16 | (upper & 0o40000);

        self.write_s15(idx, upper);
        self.write_s15(idx + 1, lower);
    }

    pub fn read_io(&mut self, idx: usize) -> u16 {
        self.mem.read_io(idx)
    }

    pub fn write_io(&mut self, idx: usize, val: u16) {
        self.mem.write_io(idx, val);
    }

    fn is_overflow(&mut self) -> bool {
        let a = self.read(REG_A);
        match a & 0xC000 {
            0xC000 | 0x0000 => false,
            _ => true,
        }
    }

    fn rupt_disabled(&mut self) -> bool {
        if self.ec_flag == true || self.gint == false || self.is_irupt == true || self.is_overflow()
        {
            return true;
        }
        return false;
    }

    fn rupt_pending(&self) -> bool {
        // Check to see if we have any interrupts to handle
        if self.rupt != 0 {
            return true;
        }
        return false;
    }

    fn handle_rupt(&mut self) {
        debug!("Interrupt Mask: {:x}", self.rupt);
        for i in 0..10 {
            let mask = 1 << i;
            if self.rupt & mask != 0 {
                // Set the interrupt flag to pending
                self.gint = false;

                // Store registers to Save State
                let val = self.read(REG_PC) + 1;
                self.write(REG_PC_SHADOW, val);
                self.write(REG_IR, self.calculate_instr_data());
                self.idx_val = 0;

                // Change the PC to specific interrupt handler code
                let new_pc = 0x800 + (i * 4);
                self.update_pc(new_pc);

                // Return
                self.rupt ^= mask;
                break;
            }
        }
    }

    pub fn execute(&mut self, inst: &AgcInst) -> bool {
        // Handle TC TRAP Instruction Counting. The way this will be implemented
        // based on the wording is to count how many continuous TC/TCF
        // instructions we receive. If it surpasses the threshold, to cause a
        // reset. The wording in Memo #260 Revision B goes...
        // "Occurs if too many consecutive TC or TCF  instructions are run..."
        match inst.mnem {
            AgcMnem::TC | AgcMnem::TCF => {
                self.non_tc_count = 0;
                self.tc_count += 1;
            }
            _ => {
                self.tc_count = 0;
                self.non_tc_count += 1;
            }
        }

        match inst.mnem {
            AgcMnem::AD => return self.ad(&inst),
            AgcMnem::ADS => return self.ads(&inst),
            AgcMnem::AUG => return self.aug(&inst),
            AgcMnem::BZF => return self.bzf(&inst),
            AgcMnem::BZMF => return self.bzmf(&inst),
            AgcMnem::CA => return self.ca(&inst),
            AgcMnem::CCS => return self.ccs(&inst),
            AgcMnem::CS => return self.cs(&inst),
            AgcMnem::DAS => return self.das(&inst),
            AgcMnem::DCA => return self.dca(&inst),
            AgcMnem::DCS => return self.dcs(&inst),
            AgcMnem::DIM => return self.dim(&inst),
            AgcMnem::DXCH => return self.dxch(&inst),
            AgcMnem::DV => return self.dv(&inst),
            AgcMnem::EXTEND => {
                self.cycles = 1;
                self.ec_flag = true;
                self.idx_val = 0x0;
            }
            AgcMnem::INCR => return self.incr(&inst),
            AgcMnem::INDEX => {
                self.cycles = 2;
                let bits = if inst.is_extended() {
                    inst.get_data_bits()
                } else {
                    inst.get_data_bits() & 0o1777
                };
                self.idx_val = self.read(inst.get_data_bits() as usize);
                self.check_editing(bits as usize);
            }
            AgcMnem::INHINT => return self.inhint(&inst),
            AgcMnem::LXCH => return self.lxch(&inst),
            AgcMnem::MASK => return self.mask(&inst),
            AgcMnem::MP => return self.mp(&inst),
            AgcMnem::MSU => return self.msu(&inst),
            AgcMnem::QXCH => return self.qxch(&inst),
            AgcMnem::RELINT => return self.relint(&inst),
            AgcMnem::RESUME => return self.resume(&inst),
            AgcMnem::ROR => return self.ror(&inst),
            AgcMnem::RAND => return self.rand(&inst),
            AgcMnem::READ => return self.read_instr(&inst),
            AgcMnem::RXOR => return self.rxor(&inst),
            AgcMnem::SU => return self.su(&inst),
            AgcMnem::TC => return self.tc(&inst),
            AgcMnem::TCF => return self.tcf(&inst),
            AgcMnem::TS => return self.ts(&inst),
            AgcMnem::WAND => return self.wand(&inst),
            AgcMnem::WOR => return self.wor(&inst),
            AgcMnem::WRITE => {
                return self.write_instr(&inst);
            }
            AgcMnem::XCH => return self.xch(&inst),
            _ => {
                warn!("Unimplemented Execution of Instruction: {:?}", inst.mnem);
                self.ec_flag = false;
                self.idx_val = 0x0;
            }
        }

        // Return true, which tells the CPU to update to the next
        // PC address. Unless its a control flow instruction
        true
    }

    pub fn print_state(&mut self) {
        info!("=========================================================");
        //debug!("AGCCpu State:");
        info!(
            "A: {:04x} L: {:04x} Q: {:04x} EB: {:04x} FB: {:04x} Z: {:04x} BB: {:04x}",
            self.read(0),
            self.read(1),
            self.read(2),
            self.read(3),
            self.read(4),
            self.read(5),
            self.read(6)
        );
        info!(
            "A': {:04x} L': {:04x} Q': {:04x} Z': {:04x} BB': {:04x} IR: {:04x}",
            self.read(8),
            self.read(9),
            self.read(0xA),
            self.read(0xD),
            self.read(0xE),
            self.read(0xF)
        );
        info!("IntMask: {:x} {:?}", self.rupt, self.gint);
        info!("IR: {:x} | INDEX: {:x}", self.ir, self.idx_val);
    }

    fn handle_ruptlock(&mut self) {
        match self.is_irupt {
            true => {
                if self.ruptlock_count < 0 {
                    self.ruptlock_count = 0;
                }

                self.ruptlock_count += self.cycles as i32;
                if self.ruptlock_count > RUPT_LOCK_COUNT {
                    debug!("RUPTLOCK Restart. Sending GOJ");
                    self.set_unprog_seq(AgcUnprogSeq::GOJ);
                }
            },
            false => {
                if self.ruptlock_count > 0 {
                    self.ruptlock_count = 0;
                }

                self.ruptlock_count -= self.cycles as i32;
                if self.ruptlock_count < -RUPT_LOCK_COUNT {
                    debug!("RUPTLOCK Restart. Sending GOJ");
                    self.set_unprog_seq(AgcUnprogSeq::GOJ);
                }

            }
        }
    }

    fn handle_nightwatch(&mut self) {
        self.nightwatch_cycles += self.cycles as u32;
        if self.nightwatch_cycles >= NIGHTWATCH_TIME {
            trace!("Checking Nightwatchman {:?}", self.nightwatch);
            self.nightwatch_cycles = 0;

            // Check to see if there was any accesses to the
            // NIGHT WATCHMAN register. If there was, continue on.
            // If not, then reboot the AGC
            if self.nightwatch == 0 {
                // Send GOJAM unprogram to restart the AGC.
                debug!("NIGHT WATCHMAN Restart. Sending GOJ");
                self.set_unprog_seq(AgcUnprogSeq::GOJ);
            }

            self.nightwatch = 0;
        }
    }

    fn handle_tc_trap(&mut self) {
        if self.tc_count >= TCMONITOR_COUNT {
            self.tc_count = 0;

            // Send GOJAM unprogram to restart the AGC.
            debug!("TC TRAP Restart. Sending GOJ");
            self.set_unprog_seq(AgcUnprogSeq::GOJ);
        } else if self.non_tc_count >= TCMONITOR_COUNT {
            self.non_tc_count = 0;

            // Send GOJAM unprogram to restart the AGC.
            debug!("TC TRAP Restart. Sending GOJ");
            self.set_unprog_seq(AgcUnprogSeq::GOJ);
        }
    }

    fn update_cycles(&mut self) {
        self.mct_counter += self.cycles as f64 * 12.0;

        self.total_cycles += self.cycles as usize;
        debug!("TotalCyles: {:?}", self.total_cycles * 12);

        self.handle_nightwatch();
        self.handle_tc_trap();
        self.handle_ruptlock();

        let timers = self.mem.fetch_timers();
        self.rupt |= timers.pump_mcts(self.cycles, &mut self.unprog);
    }

    fn step_unprogrammed(&mut self) -> bool {
        let instr = self.unprog.pop_front().unwrap();
        self.cycles = match instr {
            AgcUnprogSeq::GOJ => 2,
            AgcUnprogSeq::TCSAJ => 2,
            AgcUnprogSeq::STORE => 2,
            AgcUnprogSeq::FETCH => 2,
            AgcUnprogSeq::RUPT => 2, // TODO: This should be 3 MCT, per page 44
            // but matching with yaAGC for testing
            // (https://www.ibiblio.org/apollo/hrst/archive/1029.pdf)
            _ => 1,
        };

        // Update Timers based on instruction MCTs
        self.update_cycles();

        match instr {
            AgcUnprogSeq::GOJ => {
                self.handle_goj();
                return true;
            }
            _ => {}
        };

        if !self.rupt_disabled() {
            self.rupt |= self.mem.check_interrupts();
            if self.rupt_pending() == true {
                debug!("Handling Interrupt: {:?} {:x}", self.gint, self.rupt);
                self.handle_rupt();
                self.is_irupt = true;

                self.unprog.push_back(AgcUnprogSeq::RUPT);
                let inst_data = self.calculate_instr_data();

                self.print_state();

                let addr: usize = (self.read(REG_PC) & 0xFFFF) as usize;
                let i = disasm(addr as u16, inst_data).unwrap();
                debug!("{:x?}++++", i);

                return true;
            }
        }

        false

        //// Check for any interrupts to service at the beginning of
        //// the step process, if we are allowed
        //self.print_state();
        //
        //let mut inst_data = self.ir + self.idx_val;
        //if self.ec_flag {
        //    inst_data = inst_data | 0x8000;
        //}
        //
        //let addr : usize = ((self.read(REG_PC) & 0xFFFF)) as usize;
        //let i = disasm(addr as u16, inst_data).unwrap();
        //info!("{:x?}------", i);
    }

    fn step_programmed(&mut self) {
        // Check for any interrupts to service at the beginning of
        // the step process, if we are allowed
        self.print_state();

        if !self.rupt_disabled() {
            if self.rupt_pending() == true {
                debug!("Handling Interrupt: {:?} {:x}", self.gint, self.rupt);
                self.handle_rupt();
                self.is_irupt = true;

                self.unprog.push_back(AgcUnprogSeq::RUPT);
                let inst_data = self.calculate_instr_data();

                let addr: usize = (self.read(REG_PC) & 0xFFFF) as usize;
                let i = disasm(addr as u16, inst_data).unwrap();
                debug!("{:x?}++++", i);

                return;
            }
        }

        // Check for any interrupts to service at the beginning of
        // the step process, if we are allowed
        //self.print_state();

        let inst_data = self.calculate_instr_data();

        let addr: usize = (self.read(REG_PC) & 0xFFFF) as usize;
        let i = disasm(addr as u16, inst_data).unwrap();
        //if addr > 0x800 {
        //    println!("Address: {:o} - {:x?}", addr, i);
        //} else {
        //    println!("Address: {:o},{:o} - {:x?}", self.read(REG_FB) >> 10, addr, i);
        //}

        let next_pc = ((addr + 1) & 0xFFFF) as u16;
        //self.reg_write(REG_PC, next_pc);
        //self.ir = self.read(next_pc as usize);
        self.update_pc(next_pc);

        //debug!("PC: {:04x} {:04x} {:04x}", addr, self.ir, self.idx_val);
        self.idx_val = 0;

        if self.ec_flag {
            match i.mnem {
                AgcMnem::INDEX => {}
                _ => {
                    self.ec_flag = false;
                }
            }
        }

        //self.ir = self.read(next_pc as usize);
        match self.execute(&i) {
            true => {}
            false => {}
        }

        self.update_cycles();
    }

    pub fn step(&mut self) -> u16 {
        // Check to see if we have an unprogrammed sequence instruction
        // that was performed. If we did, create a bubble before executing
        while self.unprog.len() > 0 {
            //info!("StepUnprogrammed");
            if self.step_unprogrammed() {
                return self.cycles;
            }
        }

        // Execute Stepped programm and update timers based
        // on instruction MCTs
        self.step_programmed();
        self.cycles
    }
}

#[cfg(test)]
mod cpu_tests {
    use crate::cpu;
    use crate::instr::tests::init_agc;

    ///
    /// ## READ_DP() Unit test - RAM Address
    ///
    /// The following test will perform specific corner case testing of loading
    /// (reading) a Double Precision value from a specific location in RAM. The
    /// test will verify the functionality of both WORD with same and different
    /// sign values.
    ///
    #[test]
    fn cpu_read_dp_ram_tests() {
        let mut cpu = init_agc();
        let setup = [
            // Standard 14bit | 14 bit with the same positive and negative signs
            (0o37777, 0o37777, 0o1777777777),
            // Testing Positive Upper and Negative Lower Values and their
            // double positive counterpart
            (0o37777, 0o40000, 0o1777700001),
            (0o37776, 0o00001, 0o1777700001),
            (0o17777, 0o40000, 0o777700001),
            (0o17776, 0o00001, 0o777700001),
            // Testing Standard Negative / Negative Signs
            (0o77777, 0o77777, 0o3777777777),
            (0o67777, 0o77777, 0o3377777777),
            // TODO: Testing Negative Upper and Positive Lower values and their
            // double negative counterpart
            //(0o67776, 0o00000, 0o3377777777),
            //(0o67777, 0o77777, 0o3377777777),
            //(0o70000, 0o40000, 0o3400000000),  // TODO: Validate this somehow
            //(0o67777, 0o00000, 0o3400000000),  // TODO: Validate this somehow

            // TODO: Positive and negative zeros
            //(0o77777, 0o00000, 0o0),          // Should be positive zero
            //(0o00000, 0o00000, 0o0),          // Should be positive zero
            //(0o77777, 0o77777, 0o3777777777), // Should be negative zero
            //(0o00000, 0o77777, 0o3777777777), // Should be negative zero
        ];

        for (rega, regl, expect_val) in setup.iter() {
            cpu.reset();
            cpu.write(0o200, *rega);
            cpu.write(0o201, *regl);
            println!("read_dp: Testing {:o} {:o} => {:o}", rega, regl, expect_val);
            assert_eq!(cpu.read_s15(0o200), *rega);
            assert_eq!(cpu.read_s15(0o201), *regl);

            let a = cpu.read_dp(0o200);
            println!("Reading: {:o}", a);
            assert_eq!(a, *expect_val);
        }
    }

    /// ## READ_DP() Unit test - REG Address
    ///
    /// The following test will perform specific corner case testing of loading
    /// (reading) a Double Precision value from a specific location in register
    /// space. The test will verify the functionality of both WORD with same and
    /// different sign values, and the use of a 16-bit register
    #[test]
    fn cpu_read_dp_rega_tests() {
        let mut cpu = init_agc();
        let setup = [
            // Standard 14bit | 14 bit with the same positive and negative signs
            (0o37777, 0o37777, 0o1777777777),
            // Testing Positive Upper and Negative Lower Values and their
            // double positive counterpart
            (0o37777, 0o40000, 0o1777700001),
            (0o37776, 0o00001, 0o1777700001),
            (0o17777, 0o40000, 0o777700001),
            (0o17776, 0o00001, 0o777700001),
            // Testing Standard Negative / Negative Signs
            (0o77777, 0o77777, 0o3777777777),
            (0o67777, 0o77777, 0o3377777777),
            // Testing Negative Upper and Positive Lower values and their
            // double negative counterpart
            //(0o67777, 0o00000, 0o3377777777),
            //(0o67777, 0o77777, 0o3377777777),
            //(0o70000, 0o40000, 0o3400000000), // TODO: Validate this somehow
            //(0o67777, 0o00001, 0o3400000000), // TODO: Validate this somehow
        ];

        for (rega, regl, expect_val) in setup.iter() {
            cpu.reset();
            cpu.write_s15(cpu::REG_A, *rega);
            cpu.write_s15(cpu::REG_L, *regl);
            println!("read_dp: Testing {:o} {:o} => {:o}", rega, regl, expect_val);
            assert_eq!(cpu.read_s15(cpu::REG_A), *rega, "Readback failed for REGA");
            assert_eq!(cpu.read_s15(cpu::REG_L), *regl, "Readback failed for REGL");
            let a = cpu.read_dp(cpu::REG_A);
            //println!("Reading: {:o}", a);
            assert_eq!(a, *expect_val, "Readback failed for read_dp");
        }
    }

    /// ## WRITE_DP() Unit test - REG Address
    ///
    /// The following test will perform specific corner case testing of writing
    /// a Double Precision value to a specific location in register space. The
    /// test will verify the use of a 16-bit register
    #[test]
    fn cpu_write_dp_rega_tests() {
        let mut cpu = init_agc();
        let setup = [
            // Standard 14bit | 14 bit with the same positive and negative signs
            (0o37777, 0o37777, 0o1777777777),
            (0o177777, 0o77777, 0o3777777777),
        ];

        for (expect_a, expect_l, val) in setup.iter() {
            cpu.reset();
            cpu.write_dp(cpu::REG_A, *val);
            println!(
                "read_dp: Testing {:o} => {:o} {:o}",
                *val, *expect_a, *expect_l
            );
            assert_eq!(cpu.read(cpu::REG_A), *expect_a);
            assert_eq!(cpu.read(cpu::REG_L), *expect_l);
            assert_eq!(*val, cpu.read_dp(cpu::REG_A));
        }
    }

    #[test]
    fn cpu_test_tc_trap_reset_light() {
        let mut cpu = init_agc();

        let dur = std::time::Duration::from_secs(5);
        std::thread::sleep(dur);
        println!("Restarting AGC. Should indicate a RESTART light");
        cpu.restart();
        std::thread::sleep(dur);
    }

    #[test]
    fn cpu_test_ruptlock_restart() {
        let mut cpu = init_agc();

        cpu.write(0o04000, 0o04001);
        for i in 1..40 {
            cpu.write(0o04000 + i, 0o24000);
        }
        cpu.write(0o04000 + 40, 0o04001);
        cpu.restart();

        cpu.is_irupt = true;
        for _i in 0..12923 {
            cpu.step();
            assert_eq!(cpu.is_irupt, true);
        }

        cpu.step();
        assert_eq!(0o4000, cpu.read(super::REG_PC));
        assert_eq!(cpu.is_irupt, false);
    }
}
