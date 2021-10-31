use crate::cpu;
use crate::cpu::AgcUnprogSeq;
use crate::mem::AgcMemType;

use heapless::Deque;

use log::{debug, error};

#[derive(Clone)]
pub struct AgcTimers {
    time6_enable: bool,
    mct_counter: u16,
    timer_counter: u8,

    // Scaler
    scaler: u32,
    pub scaler_mcts: u16,
    downrupt: u32,
    downrupt_flags: u8,

    // Timer Values
    timer1: u32,
    timer3: u16,
    timer4: u16,
    timer5: u16,
    timer6: u16,
}

#[allow(dead_code)]
pub enum TimerType {
    TIME1,
    TIME2,
    TIME3,
    TIME4,
    TIME5,
    TIME6,
}

pub const MM_TIME2: usize = 0o24;
pub const MM_TIME1: usize = 0o25;
pub const MM_TIME3: usize = 0o26;
pub const MM_TIME4: usize = 0o27;
pub const MM_TIME5: usize = 0o30;
pub const MM_TIME6: usize = 0o31;

fn push_unprog_seq(unprog: &mut Deque<AgcUnprogSeq, 8>, seq: AgcUnprogSeq) {
    match unprog.push_back(seq) {
        Err(x) => {
            error!("Unable to push {:?} into UnprogSeq Deque", x);
        }
        _ => {}
    }
}

///
/// 0ms              2.5ms             5ms              7.5ms             10ms
/// |                 |                 |                 |                 |
/// |                 |                 |                 |                 |
/// |--------|--------|--------|--------|--------|--------|--------|--------|
/// |                 |                 |                 |                 |
/// TIME5           TIME4
impl AgcTimers {
    pub fn new() -> Self {
        Self {
            // Internal Counters to keep state of what is going on
            // with the timers
            mct_counter: 0,
            timer_counter: 0,
            downrupt: 1,
            downrupt_flags: 0,
            time6_enable: false,

            // scaler
            scaler: 0,
            scaler_mcts: 0,

            // Timer values
            timer1: 0,
            timer3: 0,
            timer4: 0,
            timer5: 0,
            timer6: 0,
        }
    }

    ///
    /// ## `set_downrupt_flags` Function
    ///
    /// To mimic what yaAGC implements for the DOWNRUPT logic, the following
    /// function allows the IO memory space to send flags to the timer module
    /// on when the two DOWNRUPT words are written to hardware to be sent.
    ///
    /// ### Parameters
    ///
    ///  - `flags` - The following flags are used:
    ///     - 0x1 signals DOWNRUPT1 was written to
    ///     - 0x2 signals DOWNRUPT2 was written to
    ///
    pub fn set_downrupt_flags(&mut self, flags: u8) {
        self.downrupt_flags |= flags;
        if self.downrupt_flags == 0x3 {
            self.downrupt_flags = 0x0;
            self.downrupt = 0;
        }
    }

    fn increment_scaler(&mut self, unprog: &mut Deque<AgcUnprogSeq, 8>) -> u16 {
        let mut interrupt_mask = 0;

        self.scaler += 1;
        interrupt_mask |= match self.scaler & 0o37 {
            // At every 5ms offset of timer1 and timer3, timer5 is incremented.
            // Because of this.
            // Main timer + 5ms (Timer5)
            0 => {
                debug!("SCALAR: TIMER5 Update");
                push_unprog_seq(unprog, AgcUnprogSeq::PINC);
                self.handle_timer5()
            }
            // Main Timer + 7.5ms (Timer4)
            8 => {
                debug!("SCALAR: TIMER4 Update");
                push_unprog_seq(unprog, AgcUnprogSeq::PINC);
                self.handle_timer4()
            }
            // Main timer + 10ms (Timer1 / Timer3)
            16 => {
                debug!("SCALAR: TIMER1/3 Update");
                push_unprog_seq(unprog, AgcUnprogSeq::PINC);
                push_unprog_seq(unprog, AgcUnprogSeq::PINC);
                self.handle_timer1_timer3(unprog)
            }

            // Represents the offset of 2.5ms
            _ => 0,
        };

        // Handle the TIME6 case differently. Every two scalers increments a
        // TIME6 counter
        if self.time6_enable {
            // If the timer is enabled, start processing the increment or decrement
            // of the counter
            if self.scaler % 2 == 0o00000 {
                // For every two scaler counts, handle a TIME6 DINC instruction
                // to handle.
                if self.timer6 == 0o77777 || self.timer6 == 0o00000 {
                    // If we are hitting the case where we got to zero, disable
                    // the timer and send the interrupt mask.
                    self.time6_enable = false;
                    interrupt_mask |= 1 << cpu::RUPT_TIME6;
                } else {
                    // Otherwise, we do an ABS value decrement of TIME6 register.
                    // Per the documentation.
                    push_unprog_seq(unprog, AgcUnprogSeq::DINC);
                    if self.timer6 & 0o40000 == 0o40000 {
                        self.timer6 += 1;
                    } else {
                        self.timer6 -= 1;
                    }
                }
            }
        };

        // Return the current interrupt mask value
        interrupt_mask
    }

    pub fn pump_mcts(&mut self, mcts: u16, unprog: &mut Deque<AgcUnprogSeq, 8>) -> u16 {
        let mut rupt = 0;
        debug!("SCALARcounter: {:?}", self.scaler_mcts);
        self.scaler_mcts += mcts * 3;

        // Increment the internal DOWNRUPT counter and fire every 20ms. This
        // approach allows for us to do DOWNRUPTs that is identical with yaAGC
        // for instruction comparison.
        self.downrupt += mcts as u32;
        if self.downrupt >= 1706 {
            self.downrupt = 0;
            rupt |= self.handle_downrupt();
        }

        // Handle the rest of the timers to see if any of the timers need to be
        // fired.
        rupt |= if self.scaler_mcts >= 80 {
            self.scaler_mcts -= 80;
            self.increment_scaler(unprog)
        } else {
            0
        };

        // Return the interrupt mask to the CPU so it knows what to do next.
        // Majority of the time, it should continue on, but every so often, an
        // interrupt is signaled.
        rupt
    }

    pub fn handle_timer4(&mut self) -> u16 {
        self.timer4 = (self.timer4 + 1) & 0o77777;
        if self.timer4 == 0o40000 {
            self.timer4 = 0;
            return 1 << cpu::RUPT_TIME4;
        }

        0
    }

    pub fn handle_timer5(&mut self) -> u16 {
        self.timer5 = (self.timer5 + 1) & 0o77777;
        if self.timer5 == 0o40000 {
            self.timer5 = 0;
            return 1 << cpu::RUPT_TIME5;
        }

        0
    }

    ///
    /// # Description
    ///
    /// Setter function to either enable or disable TIME6 within the TIMER
    /// peripherial module.
    ///
    /// # Arguments
    ///
    ///  - `val` - bool - Boolean to either enable (true) or disable (false)
    ///           TIME6 hardware. This will be used for an IO register later
    ///           on.
    ///
    pub fn set_time6_enable(&mut self, val: bool) {
        self.time6_enable = val;
    }

    ///
    /// # Description
    ///
    /// Setter function to either enable or disable TIME6 within the TIMER
    /// peripherial module.
    ///
    /// # Return Code
    ///
    ///  - bool - Boolean to either enable (true) or disable (false)
    ///           TIME6 hardware.
    ///
    pub fn get_time6_enable(&self) -> bool {
        return self.time6_enable;
    }

    pub fn handle_downrupt(&mut self) -> u16 {
        //self.dnrupt_counter += 1;
        //if self.dnrupt_counter % 2 == 0 {
        return 1 << cpu::RUPT_DOWNRUPT;
        //}
        //0
    }

    pub fn handle_timer1_timer3(&mut self, unprog: &mut Deque<AgcUnprogSeq, 8>) -> u16 {
        self.timer1 += 1;
        if self.timer1 & 0o37777 == 0o00000 {
            push_unprog_seq(unprog, AgcUnprogSeq::PINC);
        }

        self.timer3 = (self.timer3 + 1) & 0o77777;
        debug!("New TIMER3: {:o}", self.timer3);
        if self.timer3 == 0o40000 {
            self.timer3 = 0;
            debug!("New TIMER3 interrupt!");
            return 1 << cpu::RUPT_TIME3;
        }

        0
    }

    pub fn set_time_value(&mut self, timer_id: TimerType, value: u16) {
        match timer_id {
            TimerType::TIME1 => {
                self.timer1 = value as u32;
            }
            TimerType::TIME2 => {
                self.timer1 = value as u32;
            }
            TimerType::TIME3 => {
                self.timer3 = value & 0o77777;
            }
            TimerType::TIME4 => {
                self.timer4 = value & 0o77777;
            }
            TimerType::TIME5 => {
                self.timer5 = value & 0o77777;
            }
            TimerType::TIME6 => {
                self.timer6 = value & 0o77777;
            }
        };
    }

    ///
    /// ## read_scalar Function
    ///
    /// Provides the ability to read out the value of the current scalar value
    /// within the timer. This can be used to read for HI/LOSCALAR reads
    ///
    /// ### Paramaters
    ///
    ///  - None
    ///
    /// ### Result
    ///
    ///   - `u32` value that contains the SCALAR value that is composed of both
    ///     HISCALAR and LOSCALAR
    ///
    pub fn read_scalar(&self) -> u32 {
        self.scaler
    }

    ///
    /// ## `reset` Function
    ///
    /// Provides a method to reset the Timer module to set the appropriate values
    /// to their reset state.
    ///
    /// ### Parameters
    ///
    ///  - None
    ///
    /// ### Result
    ///
    ///  -  None
    ///
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.timer1 = 0;
        self.timer3 = 0;
        self.timer4 = 0;
        self.timer5 = 0;
        self.timer6 = 0;
    }
}

impl AgcMemType for AgcTimers {
    fn read(&self, _bank_idx: usize, bank_offset: usize) -> u16 {
        let res = match bank_offset {
            MM_TIME2 => ((self.timer1 >> 14) & 0o37777) as u16,
            MM_TIME1 => (self.timer1 & 0o37777) as u16,
            MM_TIME3 => self.timer3,
            MM_TIME4 => self.timer4,
            MM_TIME5 => self.timer5,
            MM_TIME6 => self.timer6,
            _ => 0,
        };
        debug!("Reading TIMER: {:o} = {:o}", bank_offset, res);
        res
    }

    fn write(&mut self, _bank_idx: usize, bank_offset: usize, value: u16) {
        debug!(
            "Timers: Setting {:x} to bank_offet: {:o}",
            value, bank_offset
        );
        match bank_offset {
            MM_TIME2 => {
                self.set_time_value(TimerType::TIME1, value);
            }
            MM_TIME1 => {
                self.set_time_value(TimerType::TIME1, value);
            }
            MM_TIME3 => {
                self.set_time_value(TimerType::TIME3, value);
            }
            MM_TIME4 => {
                self.set_time_value(TimerType::TIME4, value);
            }
            MM_TIME5 => {
                self.set_time_value(TimerType::TIME5, value);
            }
            MM_TIME6 => {
                self.set_time_value(TimerType::TIME6, value);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod timer_modules_tests {
    use crate::mem::timer;
    use crate::mem::AgcMemType;

    ///
    /// ## Timer Reset Test
    ///
    /// A simple test to demonstrate that a timer reset will set the timers to
    /// 0, when their prior value was non-zero.
    ///
    #[test]
    fn timer_reset_test() {
        let mut timer_mod = timer::AgcTimers::new();
        let timers_addr = [
            timer::MM_TIME1,
            timer::MM_TIME3, //timer::MM_TIME2,
            timer::MM_TIME4,
            timer::MM_TIME5,
            timer::MM_TIME6,
        ];

        // Set a known value into the timers so we can compare what it is
        // prior to a reset
        for i in timers_addr.iter() {
            timer_mod.write(0, *i, 0o11111);
            assert_eq!(timer_mod.read(0, *i), 0o11111);
        }

        // Reset the CPU, which should reset the timers. The timers should then
        // be set to their default value, which is 0o00000 currently.
        timer_mod.reset();
        for i in timers_addr.iter() {
            assert_eq!(timer_mod.read(0, *i), 0o00000);
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// Test is to do a basic test to ensure for every 10ms, all the timers are
    /// incrementing on that basis. This does not take into consideration the
    /// phase of the timer. For now, show for every 10ms over 50ms that the
    /// timer is being incremented.
    ///
    fn timer_pump_test() {
        let mut timers = super::AgcTimers::new();
        let mut unprog = heapless::Deque::new();

        for time_idx in 1..=5 {
            for _i in 0..855 {
                timers.pump_mcts(1, &mut unprog);
            }

            assert_eq!(
                timers.read(0, super::MM_TIME1),
                time_idx,
                "TIME1 did not count properly"
            );
            assert_eq!(
                timers.read(0, super::MM_TIME2),
                0,
                "TIME2 is not the right value"
            );
            assert_eq!(
                timers.read(0, super::MM_TIME3),
                time_idx,
                "TIME3 did not count properly"
            );
            assert_eq!(
                timers.read(0, super::MM_TIME4),
                time_idx,
                "TIME4 did not count properly"
            );
            assert_eq!(
                timers.read(0, super::MM_TIME5),
                time_idx,
                "TIME5 did not count properly"
            );
            assert_eq!(
                timers.read(0, super::MM_TIME6),
                0,
                "TIME6 is not disabled when expected to be"
            );
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// Test is to do a basic test to demonstrate an overflow from TIME1 to TIME2.
    /// The test will ensure that:
    ///   - Additional PINC will be added to the 10ms increment
    ///   - TIME2 is incremented by 1.
    ///
    fn test_time1_overflow_increment() {
        let mut timers = super::AgcTimers::new();
        let mut unprog = heapless::Deque::new();

        timers.write(0, super::MM_TIME1, 0o37777);
        assert_eq!(
            timers.read(0, super::MM_TIME1),
            0o37777,
            "TIME1 is not being properly set intitially before the test"
        );
        assert_eq!(
            timers.read(0, super::MM_TIME2),
            0,
            "TIME2 is not being properly set initially before the test"
        );

        for _i in 0..855 {
            timers.pump_mcts(1, &mut unprog);
        }

        assert_eq!(
            timers.read(0, super::MM_TIME1),
            0o00000,
            "TIME1 did not count properly"
        );
        assert_eq!(
            timers.read(0, super::MM_TIME2),
            0o00001,
            "TIME2 is not the right value"
        );
        assert_eq!(unprog.len(), 5, "Got the additional PINC for the overflow");
    }

    ///
    /// # Description
    ///
    /// Helper function to test a given timer that it will properly overflow
    /// and generate the associating interrupt flag.
    ///
    fn test_time_overflow(time_idx: usize, interrupt_number: u8) {
        let mut timers = super::AgcTimers::new();
        let mut unprog = heapless::Deque::new();

        timers.write(0, time_idx, 0o37777);
        assert_eq!(
            timers.read(0, time_idx),
            0o37777,
            "TIME is not being properly set intitially before the test"
        );

        let mut interrupt_flags = 0x0;
        for _i in 0..855 {
            interrupt_flags |= timers.pump_mcts(1, &mut unprog);
        }

        assert_eq!(
            timers.read(0, time_idx),
            0o00000,
            "TIMER did not count properly"
        );
        assert_eq!(unprog.len(), 4, "Got the appropriate number of PINCs");
        let flag_mask = 1 << interrupt_number;
        assert_eq!(
            interrupt_flags & flag_mask,
            flag_mask,
            "Did not receive interrupt flag"
        );
    }

    #[test]
    ///
    /// # Description
    ///
    /// Test to ensure that TIME3 is being properly overflowed and an interrupt
    /// flag is generated because of this.
    ///
    fn test_time3_overflow() {
        test_time_overflow(super::MM_TIME3, crate::cpu::RUPT_TIME3);
    }

    #[test]
    ///
    /// # Description
    ///
    /// Test to ensure that TIME4 is being properly overflowed and an interrupt
    /// flag is generated because of this.
    ///
    fn test_time4_overflow() {
        test_time_overflow(super::MM_TIME4, crate::cpu::RUPT_TIME4);
    }

    #[test]
    ///
    /// # Description
    ///
    /// Test to ensure that TIME5 is being properly overflowed and an interrupt
    /// flag is generated because of this.
    ///
    fn test_time5_overflow() {
        test_time_overflow(super::MM_TIME5, crate::cpu::RUPT_TIME5);
    }

    #[test]
    ///
    /// # Description
    ///
    /// Tests that we are able to properly enable and disable the TIME6 value,
    /// regardless of how many MCTs are being pumped into the timer module.
    ///
    fn test_time6_enable_disable() {
        let mut timers = super::AgcTimers::new();
        let mut unprog = heapless::Deque::new();

        for _i in 1..=5 {
            for _i in 0..54 {
                timers.pump_mcts(1, &mut unprog);
            }
            assert_eq!(
                timers.read(0, super::MM_TIME6),
                0,
                "TIME6 is not disabled when expected to be"
            );
        }

        timers.set_time6_enable(true);
        timers.write(0, super::MM_TIME6, 0o7);
        for time_idx in 1..=5 {
            for _i in 0..54 {
                timers.pump_mcts(1, &mut unprog);
            }
            assert_eq!(
                timers.read(0, super::MM_TIME6),
                0o7 - time_idx,
                "TIME6 is not disabled when expected to be"
            );
        }
    }

    #[test]
    ///
    /// # Description
    ///
    /// Tests that TIME6 is properly being incremented when a positive value is
    /// set for TIME6. Also, tests that we get an interrupt flag when we hit
    /// +/- 0 value
    ///
    fn test_time6_interrupt_positive() {
        let mut timers = super::AgcTimers::new();
        let mut unprog = heapless::Deque::new();

        timers.set_time6_enable(true);
        timers.write(0, super::MM_TIME6, 0o1);
        let mut interrupt_flags = 0;
        for _i in 0..54 {
            interrupt_flags |= timers.pump_mcts(1, &mut unprog);
        }
        assert_eq!(
            timers.read(0, super::MM_TIME6),
            0,
            "TIME6 value is not what is expected."
        );
        assert_eq!(interrupt_flags, 0, "Got interrupt when not suppose to yet");
        assert_eq!(true, timers.get_time6_enable(), "TIME6 is not enabled");

        // Trigger another increment which triggers the interrupt and disables
        // the TIME6 value
        for _i in 0..54 {
            interrupt_flags |= timers.pump_mcts(1, &mut unprog);
        }
        assert_eq!(
            interrupt_flags,
            1 << crate::cpu::RUPT_TIME6,
            "Did not get interrupt"
        );
        assert_eq!(
            timers.read(0, super::MM_TIME6),
            0,
            "TIME6 value is not what is expected."
        );
        assert_eq!(false, timers.get_time6_enable(), "TIME6 is not disabled");
    }

    #[test]
    ///
    /// # Description
    ///
    /// Tests that TIME6 is properly being incremented when a negative value is
    /// set for TIME6. Also, tests that we get an interrupt flag when we hit
    /// +/- 0 value
    ///
    fn test_time6_interrupt_negative() {
        let mut timers = super::AgcTimers::new();
        let mut unprog = heapless::Deque::new();
        let mut interrupt_flags = 0;

        // Enable the timer and prime it with a given value to test when the
        // timer hits 0.
        timers.set_time6_enable(true);
        timers.write(0, super::MM_TIME6, 0o77776);

        // Pump in 54 MCTs which is equivalent to 1/1600 of a second.
        for _i in 0..54 {
            interrupt_flags |= timers.pump_mcts(1, &mut unprog);
        }

        // Test that we have properly hit -0 and did not set the interrupt value
        // yet. Also TIME6 is not disabled yet. The next increment should be
        assert_eq!(
            timers.read(0, super::MM_TIME6),
            0o77777,
            "TIME6 value is not what is expected."
        );
        assert_eq!(interrupt_flags, 0, "Got interrupt when not suppose to yet");
        assert_eq!(true, timers.get_time6_enable(), "TIME6 is not enabled");

        for _i in 0..54 {
            interrupt_flags |= timers.pump_mcts(1, &mut unprog);
        }
        assert_eq!(
            interrupt_flags,
            1 << crate::cpu::RUPT_TIME6,
            "Did not get interrupt"
        );
        assert_eq!(
            timers.read(0, super::MM_TIME6),
            0o77777,
            "TIME6 value is not what is expected."
        );
        assert_eq!(false, timers.get_time6_enable(), "TIME6 is not disabled");
    }
}
