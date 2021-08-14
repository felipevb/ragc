///
/// `overflow_correction` function handles 16 bit overflow correction.
/// This function is to handle issues with overflow conditions when it comes
/// to the few S16 bit registers/
///
/// # Arguments
///
///  - `a` - Value to overflow correct
///
/// # Results
///
///  - `u16` value which represents a `s15` signed one's compliment register.
///
pub fn overflow_correction(a: u16) -> u16 {
    let newa = match 0xC000 & a {
        0x8000 => a | 0xC000,
        0x4000 => a & 0x3FFF,
        _ => a,
    };

    // Return the 15 bits
    newa
}

///
/// `sign_extend` function handles extending a one's compliment S15 value into
/// a one's compliment S15 register.
///
/// # Arguments
///
///  - `k` is a S15 one's compliment value that will be signed extended to
///    S16 one's compliment value
///
pub fn sign_extend(k: u16) -> u16 {
    let bit = k & 0x4000;
    if bit != 0 {
        let v = k | 0x8000;
        v
    } else {
        let v = k & 0x7FFF;
        v
    }
}

///
/// Converts a S15 one's compliment value into a S15 two's compliment value
///
/// # Arguments
///
///  - `k` is a S15 one's compliment value that will be converted to S15
///  two's compliment value
///
#[allow(dead_code)]
pub fn s15_ones_to_twos(val: u16) -> u16 {
    if val & 0x4000 == 0x4000 {
        (val + 1) & 0x7FFF
    } else {
        val & 0x7FFF
    }
}

pub fn s15_add(a: u16, b: u16) -> u16 {
    let mut res = a as u32 + b as u32;
    if res & 0o100000 == 0o100000 {
        res += 1;
    }
    (res & 0o77777) as u16
}

pub fn s16_add(a: u16, b: u16) -> u16 {
    let mut res = a as u32 + b as u32;
    if res & 0xFFFF0000 != 0x00000000 {
        res += 1;
    }
    (res & 0o177777) as u16
}

pub fn _dp_add(a: u32, b: u32) -> u32 {
    let mut res = a + b;
    if res & 0xE0000000 != 0x0 {
        res += 1;
    }
    res
}

pub fn cpu_to_agc_sp(cpu_val: i16) -> u16 {
    if cpu_val <= 0 {
        !((cpu_val * -1) as u16)
    } else {
        cpu_val as u16
    }
}

pub fn agc_sp_to_cpu(agc_val: u16) -> i16 {
    if agc_val & 0o040000 != 0 {
        -(((!agc_val) & 0o037777) as i16)
    } else {
        (agc_val & 0o37777) as i16
    }
}

pub fn agc_dp_to_cpu(agc_val: u32) -> i32 {
    if agc_val & 0o2000000000 != 0 {
        -(((!agc_val) & 0o1777777777) as i32)
    } else {
        (agc_val & 0o1777777777) as i32
    }
}

pub fn generate_yaagc_packet(channel: usize, value: u16) -> [u8; 4] {
    [
        0x0 | ((channel >> 3) & 0x1F) as u8,
        0x40 | ((channel & 0x7) << 3) as u8 | ((value >> 12) & 0x7) as u8,
        0x80 | ((value >> 6) & 0x3F) as u8,
        0xC0 | (value & 0x3F) as u8,
    ]
}

pub fn parse_yaagc_packet(msg: [u8; 4]) -> Option<(u16, u16)> {
    let a;
    let b;
    let c;
    let d;

    match msg.len() {
        4 => {
            a = *msg.get(0).unwrap();
            b = *msg.get(1).unwrap();
            c = *msg.get(2).unwrap();
            d = *msg.get(3).unwrap();
        }
        5 => {
            a = *msg.get(0).unwrap();
            b = *msg.get(1).unwrap();
            c = *msg.get(2).unwrap();
            d = *msg.get(3).unwrap();
        }
        _ => {
            return None;
        }
    }

    if a & 0xC0 != 0x00 || b & 0xC0 != 0x40 || c & 0xC0 != 0x80 || d & 0xC0 != 0xC0 {
        return None;
    }

    let value: u16 = ((b as u16) & 0x7) << 12 | ((c as u16) & 0x3F) << 6 | ((d & 0x3F) as u16);
    let channel: u16 = ((a as u16) & 0x3F) << 3 | ((b as u16) >> 3 & 0x7);
    Some((channel, value))
}

#[cfg(test)]
mod utils_tests {
    use super::*;

    #[test]
    fn test_overflow_correction_pos() {
        for test_val in 0o040000..0o077777 {
            let result = overflow_correction(test_val);
            assert_eq!(test_val & 0o37777, result);
        }
    }

    #[test]
    fn test_overflow_correction_neg() {
        for test_val in 0o100000..0o137777 {
            let result = overflow_correction(test_val);
            assert_eq!(test_val | 0o40000, result);
        }
    }

    #[test]
    ///
    /// Tests the `sign_extend` function to check to see that there is no
    /// sign extension for positive values.
    ///
    fn test_sign_extend_positive() {
        for test_val in 0o000000..=0o037777 {
            let result = sign_extend(test_val);
            assert_eq!(
                test_val, result,
                "Failed sign extension: Expected: {:o} | Result: {:o}",
                test_val, result
            );
        }
    }

    #[test]
    ///
    /// Tests the `sign_extend` function to check to see that there is a proper
    /// sign extension.
    ///
    fn test_sign_extend_negative() {
        for test_val in 0o040000..=0o077777 {
            let result = sign_extend(test_val);
            assert_eq!(
                test_val | 0o100000,
                result,
                "Failed sign extension: Expected: {:o} | Result: {:o}",
                test_val,
                result
            );
        }
    }

    #[test]
    ///
    /// `s15_ones_to_twos` test to check the positive signed values
    ///   are being properly converted from one's compliment to twos
    ///   compliment.
    ///
    /// The test will check all positive values for one's complement
    /// s15 from 0o00000 to 0o37777 (bit 15 set to 10
    ///
    fn s15_ones_to_twos_test_positive() {
        for test_val in 0o00000..=0o37777 {
            let result = s15_ones_to_twos(test_val);
            assert_eq!(
                test_val, result,
                "s15_ones_to_twos failed. Expected {:o} | Result: {:o}",
                test_val, result
            );
        }
    }

    #[test]
    ///
    /// `s15_ones_to_twos` test to check the negative signed values
    ///   are being properly converted from one's compliment to twos
    ///   compliment.
    ///
    /// The test will check all negative values for one's complement
    /// s15 from 0o40000 to 0o77777 (bit 15 set to 1)
    ///
    fn s15_ones_to_twos_test_negative() {
        for test_val in 0o40000..=0o77777 {
            let result = s15_ones_to_twos(test_val);
            assert_eq!(
                (test_val + 1) & 0o77777,
                result,
                "s15_ones_to_twos failed. Expected {:o} | Result: {:o}",
                test_val,
                result
            );
        }
    }

    #[test]
    ///
    /// Testing `s15_add` function to handle all the one's compliment signed
    /// 15 bit additions.
    ///
    fn s15_add_tests() {
        let test_vals = [
            // Test the zero case
            (0o77777, 0o77777, 0o77777),
            (0, 0, 0),
            // Test the basic math cases
            (1, 1, 2),
            (0o77776, 0o77776, 0o77775),
            // IR: 47ff | INDEX: 3809 | Result: 0009
            (0x47ff, 0x3809, 0x0009),
        ];

        for (a, b, expect) in test_vals.iter() {
            let res = s15_add(*a, *b);
            assert_eq!(
                *expect, res,
                "Failed S15 Addition: {:o} + {:o} = {:o}, Result: {:o}",
                a, b, expect, res
            );
        }
    }
}
