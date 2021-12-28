use super::AgcInst;
use crate::cpu::AgcCpu;
use crate::consts::cpu::*;

use log::{debug, error, trace};

fn s15_abs(value: u16) -> u16 {
    if value & 0o40000 == 0o40000 {
        !value & 0o77777
    } else {
        value & 0o77777
    }
}

fn convert_to_dp(upper: u16, lower: u16) -> u32 {
    let zero_list = [0o77777, 0o00000];

    // If the upper is +/-0, then the sign value is dictated by the lower
    // portion of the DP value. Whichever sign it is, we just sign-extend
    if zero_list.contains(&upper) {
        // If the sign bit is set, sign extend it with 1s
        if lower & 0o40000 == 0o40000 {
            lower as u32 | 0o17777700000
        }
        // Otherwise, we just give the current value, since sign extension
        // is just zeros
        else {
            lower as u32
        }
    }
    // If the upper is +/-0, then the sign value is dictated by the lower
    // portion of the DP value. Whichever sign it is, we just sign-extend
    else {
        match (upper & 0o40000) == (lower & 0o40000) {
            true => (upper as u32) << 14 | (lower & 0o37777) as u32,
            false => {
                let mut res = if lower & 0o40000 == 0o40000 {
                    // Subtract 1 from the upper to borrow for the lower to
                    // convert to the right singed value
                    let mut val: u32 = crate::utils::s15_add(upper, 0o77776) as u32;
                    val = val << 14;

                    // Add in the lower portion into the result value
                    val |= crate::utils::s15_add(lower, 0o40000) as u32;
                    val
                } else {
                    let mut val: u32 = crate::utils::s15_add(upper, 0o00001) as u32;
                    val = val << 14;
                    val |= crate::utils::s15_add(lower, 0o37777) as u32;
                    val
                };

                if res & 0o4000000000 == 0o4000000000 {
                    res += 1;
                }
                res & 0o3777777777
            }
        }
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod convert_test {
    #[test]
    fn test_convert() {
        println!("{:o}", super::convert_to_dp(0o37777, 0o40000));
        println!("{:o}", super::convert_to_dp(0o40000, 0o37777));
        println!("{:o}", super::convert_to_dp(0o60000, 0o40000)); // 3000000000
    }

    #[test]
    fn test_abs() {
        println!("{:o}", super::s15_abs(0o37776));
        println!("{:o}", super::s15_abs(0o40001));
    }
}

pub trait AgcArith {
    fn ad(&mut self, inst: &AgcInst) -> u16;
    fn ads(&mut self, inst: &AgcInst) -> u16;
    fn das(&mut self, inst: &AgcInst) -> u16;
    fn aug(&mut self, inst: &AgcInst) -> u16;
    fn mp(&mut self, inst: &AgcInst) -> u16;

    fn su(&mut self, inst: &AgcInst) -> u16;
    fn msu(&mut self, inst: &AgcInst) -> u16;
    fn incr(&mut self, inst: &AgcInst) -> u16;
    fn dim(&mut self, inst: &AgcInst) -> u16;
    fn dv(&mut self, inst: &AgcInst) -> u16;
}

impl<'a> AgcArith for AgcCpu<'a> {
    fn ad(&mut self, inst: &AgcInst) -> u16 {
        let a = self.read_s16(REG_A) as u16;
        let k = self.read_s16(inst.get_kaddr()) as u16;

        let mut res: u32 = a as u32 + k as u32;
        if res & 0xFFFF0000 != 0 {
            res += 1;
        }

        //debug!("A: {:x} | K: {:x} = R {:x}", a, k, res);
        self.write_s16(REG_A, (res & 0xFFFF) as u16);
        self.check_editing(inst.get_kaddr());
        2
    }

    fn ads(&mut self, inst: &AgcInst) -> u16 {
        let a = self.read_s16(REG_A) as u32;
        let k = self.read_s16(inst.get_kaddr_ram());

        let mut res: u32 = a as u32 + k as u32;
        if res & 0xFFFF0000 != 0 {
            res += 1;
        }

        let newval = (res & 0xFFFF) as u16;
        self.write_s16(REG_A, newval);
        self.write_s16(inst.get_kaddr_ram(), newval);
        2
    }

    fn das(&mut self, inst: &AgcInst) -> u16 {
        let mut k = inst.get_kaddr_ram();
        if k > 0 {
            k -= 1;
        }

        let a = self.read_s16(REG_A);
        let l = self.read_s16(REG_L);

        let word1 = self.read_s16(k);
        let word2 = self.read_s16(k + 1);

        let mut res_upper = crate::utils::s16_add(a, word1);
        let mut res_lower = crate::utils::s16_add(l, word2);

        match res_lower & 0o140000 {
            0o040000 => {
                res_upper = crate::utils::s16_add(res_upper, 0o00001);
                res_lower = crate::utils::overflow_correction(res_lower);
            }
            0o100000 => {
                res_upper = crate::utils::s16_add(res_upper, 0o177776);
                res_lower = crate::utils::overflow_correction(res_lower);
            }
            _ => {}
        };

        self.write_s16(REG_L, 0);
        match res_upper & 0o140000 {
            0o040000 => {
                self.write_s16(REG_A, 0o000001);
            }
            0o100000 => {
                self.write_s16(REG_A, 0o177776);
            }
            _ => {
                self.write_s16(REG_A, 0o000000);
            }
        }

        self.write_s16(k, res_upper);
        self.write_s16(k + 1, res_lower);
        3
    }

    fn aug(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr_ram();

        match k {
            REG_A | REG_Q => {
                let v = self.read_s16(k as usize);
                let newv = match v & 0o100000 {
                    0o100000 => v - 1,
                    0o000000 => v + 1,
                    _ => {
                        error! {"This should be hit"};
                        0
                    }
                };
                self.write_s16(k as usize, newv);
            }
            _ => {
                let v = self.read_s15(k as usize);
                let newv = match v & 0o40000 {
                    0o40000 => v - 1,
                    0o00000 => v + 1,
                    _ => {
                        error! {"This should be hit"};
                        0
                    }
                };
                self.write_s15(k as usize, newv);
            }
        }

        2
    }

    ///
    /// ## MP instruction
    ///
    /// The following instruction performs arithmetic multiply between the value
    /// stored in A and in K, and the Double Precision result is stored in A/L
    ///
    /// ### Parameters
    ///  - `inst` - Agc Instruction Structure that has been disassembled
    ///
    fn mp(&mut self, inst: &AgcInst) -> u16 {
        // Fetch the A and K values in preparation to multiply. Also capture the
        // sign values of each of these so we know what the result should be.
        // Get the Sign and Magnitude Bits from A
        let a = self.read_s15(REG_A);
        let a_sign = a & 0o40000;
        let a_mag = if a_sign != 0o0 {
            (!a) & 0o37777
        } else {
            a & 0o37777
        };

        // Get the Sign and Magnitude Bits from K so we can perform basic
        // magnitude multiplication and apply the sign at the end of
        // the operation
        let k = self.read_s15(inst.get_kaddr());
        let k_sign = k & 0o40000;
        let k_mag = if k_sign != 0o0 {
            (!k) & 0o37777
        } else {
            k & 0o37777
        };

        let mut res = (a_mag as u32 * k_mag as u32) & 0o1777777777;
        if k_sign != a_sign {
            // Handle the case where we have a zero result and non-zero result.
            // With the zero result, we have more conditions to statisfy
            match res {
                // Handle the condition based on the documentation on the
                // VirtualAGC site that lists:
                //    1. The result is +0, unless
                //    2. The factor in the accumulator had been ±0 and the
                //       factor in K had been non-zero of the opposite sign,
                //       in which case the result is -0.
                0o0000000000 | 0o1777777777 => {
                    // Check to see if A is +/-0 but K != +/-0
                    // to ensure a negative zero
                    if (a_mag == 0o0 || a_mag == 0o77777) && (k_mag != 0o0 && k_mag != 0o77777) {
                        res = 0o3777777777;
                    }
                    // We have a zero result, and alternating sign, but the #2
                    // criteria did not statisfy (a == 0 && k != 0)
                    else {
                        res = 0o0000000000;
                    }
                }

                // Otherwise, follow the regular rule for signed multiplication
                // where alternating signs yield a negative result while same
                // signs yield a positive result
                _ => {
                    res = (!res) & 0o3777777777;
                }
            }
        }
        self.write_dp(REG_A, res);
        3
    }

    fn incr(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr_ram();
        let val: u32 = self.read(k) as u32;
        trace!("INCR: {:x}: {:x}", k, val);

        let kval = match k {
            REG_A | REG_Q => match val {
                0o077777 => val & 0o177777,
                0o177777 => 0o000001,
                _ => (val + 1) & 0o177777,
            },
            _ => match val {
                0o37777 => 0o00000,
                0o77777 => 0o00001,
                _ => (val + 1) & 0o77777,
            },
        };

        self.write(k, (kval & 0o177777) as u16);
        2
    }

    ///
    /// ## SU instruction
    ///
    /// The following instruction performs a subtraction of value at K from
    /// the Accumulator.
    ///
    /// ### Parameters
    ///  - `inst` - Agc Instruction Structure that has been disassembled
    ///
    fn su(&mut self, inst: &AgcInst) -> u16 {
        let a = self.read_s16(REG_A);
        let kval = !self.read_s16(inst.get_kaddr_ram());
        let mut res: u32 = a as u32 + kval as u32;
        if res & 0xFFFF0000 != 0x00000000 {
            res += 1;
        }
        self.write_s16(REG_A, (res & 0xFFFF) as u16);
        self.check_editing(inst.get_kaddr_ram());
        2
    }

    ///
    /// ## MSU instruction
    ///
    /// The following instruction performs a modular subtraction. This is
    /// performing a subtraction of two 2s-compliment values and storing the
    /// 1s compliment result into A.
    ///
    /// ### Parameters
    ///  - `inst` - Agc Instruction Structure that has been disassembled
    ///
    fn msu(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr_ram();
        match k {
            REG_A | REG_Q => {
                let kval = !self.read_s16(k);
                let aval = self.read_s16(REG_A);
                let mut res = (kval as u32 + aval as u32 + 1) & 0o177777;
                if res & 0o100000 == 0o100000 {
                    res = (res + 0o177777) & 0o177777;
                }

                trace!("MSU16: A2{:6o} - K2{:6o} = A1{:6o}", aval, kval, res);
                self.write_s16(REG_A, (res & 0o177777) as u16);
            }
            _ => {
                let kval = !self.read_s15(k) & 0o77777;
                let aval = self.read_s15(REG_A);
                let mut res = (kval + 1 + aval) & 0o77777;
                if res & 0o40000 == 0o40000 {
                    res = (res + 0o77777) & 0o77777;
                }

                trace!("MSU15: A2{:5o} - K2{:5o} = A1{:5o}", aval, kval, res);
                self.write_s15(REG_A, res);
            }
        }

        self.check_editing(k);
        2
    }

    ///
    /// ## DIM instruction
    ///
    /// The following instruction diminishes the magnitude of the value stored
    /// in K. When the value is 0(+/-), then the operation becomes a no-operation
    /// and K stays as zero.
    ///
    /// ### Parameters
    ///  - `inst` - Agc Instruction Structure that has been disassembled
    ///
    fn dim(&mut self, inst: &AgcInst) -> u16 {
        let k = inst.get_kaddr_ram();
        let kval = self.read_s16(k);
        debug!("DIM: {:x}: {:x}", k, kval);

        // Check to see what we need to do with this value. Depending on what
        // value we read, we perform different operations
        match kval {
            // If we are +/-0, then we just nop and don't alter the K value
            // From here.
            0o177777 | 0o00000 => {},

            // This means we are non-zero so we need to DIM the magnitude of
            // the value stored in K.
            _ => {
                if kval & 0o40000 == 0o40000 {
                    self.write_s16(k, kval + 1);
                } else {
                    // This is due to the rule of 1 + (-1) = -0;
                    // Because of this and since we are not doing a proper
                    // one's compliment addition, check for 0 before doing
                    // anything else
                    if kval - 1 == 0 {
                        self.write_s16(k, 0o177777);
                    } else {
                        self.write_s16(k, kval - 1);
                    }
                }
            }
        };

        2
    }

    ///
    /// ## DV instruction
    ///
    /// Implements the DV instruction which perform a division operation
    /// in the form of (A,L) / MemS15(k) = StoreDP(A,L)
    ///
    /// ### Parameters
    ///
    ///   - `inst` - Instruction that was assembled to DV instruction
    ///
    ///
    fn dv(&mut self, inst: &AgcInst) -> u16 {
        let zero_list = [0o77777, 0o00000];

        let divisor = self.read_s15(inst.get_kaddr_ram());
        let dividend_upper = self.read_s15(REG_A);
        let dividend_lower = self.read_s15(REG_L);

        // The sign of the quotient (A register) is (as usual, according to the
        // rules of arithmetic) positive if the signs of the dividend and
        // divisor agree, and is negative if the signs of the dividend and
        // divisor differ.  The sign of the remainder (L register) is the sign
        // of the dividend.
        //
        // Obtain the proper sign of the overall dividend. The true sign is
        // typically the upper, unless upper is +/-0, in which case the sign
        // is in the lower
        let divisor_sign = divisor & 0o40000;
        let dividend_sign = if zero_list.contains(&dividend_upper) {
            dividend_lower & 0o40000
        } else {
            dividend_upper & 0o40000
        };

        // Per the online documentation, handle the cases in which the dividend
        // is +/-0, if so, handle the cases mentioned below
        if zero_list.contains(&dividend_upper) && zero_list.contains(&dividend_lower) {
            if !zero_list.contains(&divisor) {
                // If the dividend is ±0 but the divisor is non-zero, the L
                // register remains unchanged and the A register is assigned 0
                // with a sign according to the rules above.
                if dividend_sign ^ divisor_sign == 0o00000 {
                    self.write_s15(REG_A, 0o00000);
                } else {
                    self.write_s15(REG_A, 0o77777);
                }
            } else {
                // If both the dividend and the divisor are ±0, the A register
                // will be stored with ±37777, and the L register will remain
                // unchanged.
                if dividend_sign ^ divisor_sign == 0o00000 {
                    self.write_s15(REG_A, 0o37777);
                } else {
                    self.write_s15(REG_A, 0o40000);
                }
            };
            return 6;
        };

        // Dividend is now non-zero at this point.
        // If the divisor is equal to the dividend in magnitude and they
        // are nonzero, then the A register will be stored with ±37777, while
        // the L register will be stored with the dividend.
        if s15_abs(dividend_upper) == s15_abs(divisor) {
            if zero_list.contains(&dividend_lower) {
                if dividend_sign ^ divisor_sign == 0o00000 {
                    self.write_s15(REG_A, 0o37777);
                } else {
                    self.write_s15(REG_A, 0o40000);
                }
                self.write_s15(REG_L, dividend_upper);
                return 6;
            } else {
                log::warn!("Undefined behavior for DV!");
            }

            return 6;
        }

        let dividend = convert_to_dp(dividend_upper, dividend_lower);
        let cpu_dividend = crate::utils::agc_dp_to_cpu(dividend);
        let cpu_divisor = crate::utils::agc_sp_to_cpu(divisor);

        let cpu_quotent = cpu_dividend / (cpu_divisor as i32);
        let cpu_remainder = cpu_dividend % (cpu_divisor as i32);

        self.write_s16(REG_A, crate::utils::cpu_to_agc_sp(cpu_quotent as i16));
        match cpu_remainder {
            0 => {
                if dividend_sign == 0o40000 {
                    self.write_s15(REG_L, 0o77777);
                } else {
                    self.write_s15(REG_L, 0o00000);
                }
            }
            _ => {
                self.write_s15(REG_L, crate::utils::cpu_to_agc_sp(cpu_remainder as i16));
            }
        }

        6
    }
}
