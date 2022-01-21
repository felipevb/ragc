//use log::error;
//use std::result;
//use crate::cpu::{AgcCpu};
//use crate::mem::AgcMemoryMap;
use crate::instr::{AgcInst, AgcMnem};

fn disasm_extended(mut i: AgcInst) -> Result<AgcInst, &'static str> {
    let opbits = i.get_opcode_bits();
    match opbits {
        0 => {
            let exb: u8 = ((i.inst_data & 0x0E00) >> 9) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    i.mnem = AgcMnem::READ;
                }
                Some(1) => {
                    i.mnem = AgcMnem::WRITE;
                    i.mct = 2;
                }
                Some(2) => {
                    i.mnem = AgcMnem::RAND;
                }
                Some(3) => {
                    i.mnem = AgcMnem::WAND;
                }
                Some(4) => {
                    i.mnem = AgcMnem::ROR;
                }
                Some(5) => {
                    i.mnem = AgcMnem::WOR;
                }
                Some(6) => {
                    i.mnem = AgcMnem::RXOR;
                }
                Some(7) => {
                    i.mnem = AgcMnem::EDRUPT;
                }
                _ => {
                    //error!(
                    //    "Invalid Extrabits Encoding for {}: {:?}",
                    //    opbits, i.extrabits
                    //);
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
            return Ok(i);
        }
        1 => {
            let exb: u8 = ((i.inst_data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    i.mnem = AgcMnem::DV;
                }
                _ => {
                    i.mnem = AgcMnem::BZF;
                }
            }
        }
        2 => {
            let exb: u8 = ((i.inst_data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    i.mnem = AgcMnem::MSU;
                }
                Some(1) => {
                    i.mnem = AgcMnem::QXCH;
                }
                Some(2) => {
                    i.mnem = AgcMnem::AUG;
                }
                Some(3) => {
                    i.mnem = AgcMnem::DIM;
                }
                _ => {
                    //error!(
                    //    "Invalid Extrabits Encoding for {}: {:?}",
                    //    opbits, i.extrabits
                    //);
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
            return Ok(i);
        }
        3 => {
            i.mnem = AgcMnem::DCA;
        }
        4 => {
            i.mnem = AgcMnem::DCS;
        }
        5 => {
            i.mnem = AgcMnem::INDEX;
        }
        6 => {
            let exb: u8 = ((i.inst_data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    i.mnem = AgcMnem::SU;
                }
                _ => {
                    i.mnem = AgcMnem::BZMF;
                }
            }
        }
        7 => {
            i.mnem = AgcMnem::MP;
        }
        _ => {
            //error!(
            //    "Invalid value found. We didn't properly mask the opcode bits. {}",
            //    opbits
            //);
            return Err("Invalid Opcode Size");
        }
    }
    Ok(i)
}

fn disasm_simple(mut i: AgcInst) -> Result<AgcInst, &'static str> {
    let opbits = i.get_opcode_bits();
    match opbits {
        0 => {
            i.mnem = match i.inst_data & 0xFFF {
                3 => AgcMnem::RELINT,
                4 => AgcMnem::INHINT,
                6 => AgcMnem::EXTEND,
                _ => AgcMnem::TC,
            };
        }
        1 => {
            let exb: u8 = ((i.inst_data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    i.mnem = AgcMnem::CCS;
                }
                Some(1) => {
                    i.mnem = AgcMnem::TCF;
                }
                Some(2) => {
                    i.mnem = AgcMnem::TCF;
                }
                Some(3) => {
                    i.mnem = AgcMnem::TCF;
                }
                _ => {
                    //error!(
                    //    "Invalid Extrabits Encoding for {}: {:?}",
                    //    opbits, i.extrabits
                    //);
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
        }
        2 => {
            let exb: u8 = ((i.inst_data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    i.mnem = AgcMnem::DAS;
                }
                Some(1) => {
                    i.mnem = AgcMnem::LXCH;
                }
                Some(2) => {
                    i.mnem = AgcMnem::INCR;
                }
                Some(3) => {
                    i.mnem = AgcMnem::ADS;
                }
                _ => {
                    //error!(
                    //    "Invalid Extrabits Encoding for {}: {:?}",
                    //    opbits, i.extrabits
                    //);
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
        }
        3 => {
            i.mnem = AgcMnem::CA;
            i.mct = 2;
        }
        4 => {
            i.mnem = AgcMnem::CS;
            i.mct = 2;
        }
        5 => {
            let exb: u8 = ((i.inst_data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            match i.extrabits {
                Some(0) => {
                    if i.inst_data & 0o07777 == 0o00017 {
                        i.mnem = AgcMnem::RESUME;
                    } else {
                        i.mnem = AgcMnem::INDEX;
                    }
                }
                Some(1) => {
                    i.mnem = AgcMnem::DXCH;
                }
                Some(2) => {
                    i.mnem = AgcMnem::TS;
                    i.mct = 2;
                }
                Some(3) => {
                    i.mnem = AgcMnem::XCH;
                }
                _ => {
                    //error!(
                    //    "Invalid Extrabits Encoding for {}: {:?}",
                    //    opbits, i.extrabits
                    //);
                    i.extrabits = None;
                    return Err("Invaid Extrabits Encoding");
                }
            }
        }
        6 => {
            i.mnem = AgcMnem::AD;
            i.mct = 2;
        }
        7 => {
            i.mnem = AgcMnem::MASK;
        }
        _ => {
            //error!(
            //    "Invalid value found. We didn't properly mask the opcode bits. {}",
            //    opbits
            //);
            return Err("Invalid Opcode Size");
        }
    }

    Ok(i)
}

pub fn disasm(pc: u16, inst_data: u16) -> Result<AgcInst, &'static str> {
    let i = AgcInst {
        pc: pc,
        inst_data: inst_data,
        mnem: AgcMnem::INVALID,
        extrabits: None,
        mct: 1,
    };

    if i.is_extended() {
        return disasm_extended(i);
    } else {
        return disasm_simple(i);
    }
}
