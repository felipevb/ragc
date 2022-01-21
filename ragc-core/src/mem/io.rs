use super::periph::engines::LmEngines;
use super::periph::AgcIoPeriph;
use crate::{utils::Option as Option};
use crate::consts::io;

//use log::{debug, error, warn};

pub struct AgcIoSpace<'a> {
    io_mem: [u16; 256],
    downrupt: Option<&'a mut dyn AgcIoPeriph>,
    dsky: Option<&'a mut dyn AgcIoPeriph>,
    lm_engines: LmEngines,
}

impl<'a> AgcIoSpace<'a> {
    pub fn new(downrupt: &'a mut dyn AgcIoPeriph,
               dsky: &'a mut dyn AgcIoPeriph) -> Self
    {
        let mut s = Self {
            io_mem: [0; 256],
            downrupt: Option::Some(downrupt),
            dsky: Option::Some(dsky),
            lm_engines: LmEngines::new(),
        };

        // Mark the engine as off to start off with
        //s.io_mem[CHANNEL_DSALMOUT] = 0x0000;
        s.io_mem[0o30] = 0o37777;
        s.io_mem[0o31] = 0o77777;
        s.io_mem[0o32] = 0o77777;
        s.io_mem[0o33] = 0o77777;
        s
    }

    pub fn blank() -> Self {
        let mut s = Self {
            io_mem: [0; 256],
            downrupt: Option::None,
            dsky: Option::None,
            lm_engines: LmEngines::new(),
        };

        // Mark the engine as off to start off with
        //s.io_mem[CHANNEL_DSALMOUT] = 0x0000;
        s.io_mem[0o30] = 0o37777;
        s.io_mem[0o31] = 0o77777;
        s.io_mem[0o32] = 0o77777;
        s.io_mem[0o33] = 0o77777;
        s
    }

    ///
    ///
    ///
    ///
    fn _handle_channel11_read(&mut self) -> u16 {
        // Get the current status of Channel 11 from the engines section
        // of the hardware bits
        let engine_enabled = if self.lm_engines._get_engine_enable() {
            1 << 12
        } else {
            1 << 13
        };

        // Rest of the bits are DSKY related bit. Mostly BIT1 through BIT7
        // where index starts at 1.
        engine_enabled & 0x3000 | self.io_mem[0o11] & 0x037F
    }

    fn handle_channel30_read(&mut self) -> u16 {
        let ae = self.lm_engines.get_active_engine();

        // For Bit2 - Get which engine is active currently via the command
        // module. If the module is set to DECEND, BIT2 is high. Otherwise,
        // it is ascend engine
        let verify_engine_bit = if ae == super::periph::engines::LM_DECEND_ENGINE {
            1 << 1
        } else {
            0o00000
        };

        0o37775 | verify_engine_bit
    }

    pub fn read(&mut self, channel_idx: usize) -> u16 {
        //debug!("IO Space Read: 0o{:o}", channel_idx);
        match channel_idx {
            // # CHANNEL 1     IDENTICAL TO COMPUTER REGISTER L (0001)
            // # CHANNEL 2     IDENTICAL TO COMPUTER REGISTER Q (0002)
            //io::CHANNEL_L => { self.parent.read(crate::cpu::REG_L) }
            //io::CHANNEL_Q => { self.parent.read(crate::cpu::REG_Q) }

            // # CHANNEL 3     HISCALAR; INPUT CHANNEL; MOST SIGNIFICANT 14 BITS FROM 33 STAGE BINARY COUNTER. SCALE
            // #               FACTOR IS B23 IN CSEC, SO MAX VALUE ABOUT 23.3 HOURS AND LEAST SIGNIFICANT BIT 5.12 SECS.
            // # CHANNEL 4     LOSCALAR; INPUT CHANNEL; NEXT MOST SIGNIFICANT 14 BITS FROM THE 33 STAGE BINARY COUNTER
            // #               ASSOCIATED WITH CHANNEL 3. SCALE FACTOR IS B9 IN  CSEC. SO MAX VAL IS 5.12 SEC AND LEAST
            // #               SIGNIFICANT BIT IS 1/3200 SEC. SCALE FACTOR OF D.P. WORD WITH CHANNEL 3 IS B23 CSEC.
            io::CHANNEL_LOSCALAR | io::CHANNEL_HISCALAR => 0,

            // # CHANNEL 7     SUPERBNK; OUTPUT CHANNEL; NOT RESET BY RESTART;   FIXED EXTENSION BITS USED TO SELECT THE
            // #               APPROPRIATE FIXED MEMORY BANK IF FBANK IS 30 OCTAL OR MORE. USES BITS 5-7.
            io::CHANNEL_SUPERBNK => self.io_mem[channel_idx] & 0o00160,

            // # CHANNEL 5     PYJETS; OUTPUT CHANNEL; PITCH RCS JET CONTROL.   (REACTION CONTROL SYSTEM) USES BITS 1-8.
            //
            // # CHANNEL 6     ROLLJETS; OUTPUT CHANNEL; ROLL RCS JET CONTROL.   (REACTION CONTROL SYSTEM) USES BIT 1-8.
            //
            // # CHANNEL 10    OUTO; OUTPUT CHANNEL; REGISTER USED TO TRANSMIT  LATCHING-RELAY DRIVING INFORMATION FOR
            // #               THE DISPLAY SYSTEM. BITS 15-12 ARE SET TO THE ROW NUMBER (1-14 OCTAL) OF THE RELAY TO BE
            // #               CHANGED AND BITS 11-1 CONTAIN THE REQUIRED SETTINGS FOR THE RELAYS IN THE ROW.
            io::CHANNEL_PYJETS | io::CHANNEL_ROLLJETS => self.io_mem[channel_idx],

            io::CHANNEL_DSKY => {
                //warn!("DSKY: Reading from DSKY value. which is weird");
                0
            }
            // # CHANNEL 11    DSALMOUT; OUTPUT CHANNEL; REGISTER WHOSE BITS ARE USED FOR ENGINE ON-OFF CONTROL AND TO
            // #               DRIVE INDIVIDUAL INDICATORS OF THE DISPLAY SYSTEM. BITS 1-7 ARE A RELAYS.
            //
            // #               BIT 1           ISS WARNING
            // #               BIT 2           LIGHT COMPUTER ACTIVITY LAMP
            // #               BIT 3           LIGHT UPLINK ACTIVITY LAMP
            // #               BIT 4           LIGHT TEMP CAUTION LAMP
            // #               BIT 5           LIGHT KEYBOARD RELEASE LAMP
            // #               BIT 6           FLASH VERB AND NOUN LAMPS
            // #               BIT 7           LIGHT OPERATOR ERROR LAMP
            // ## Page 55
            // #               BIT 8           SPARE
            // #               BIT 9           TEST CONNECTOR OUTBIT
            // #               BIT 10          CAUTION RESET
            // #               BIT 11          SPARE
            // #               BIT 12          SPARE
            // #               BIT 13          ENGINE ON
            // #               BIT 14          ENGINE OFF
            // #               BIT 15          SPARE
            io::CHANNEL_DSALMOUT => self.io_mem[io::CHANNEL_DSALMOUT], //self.handle_channel11_read(),

            // # CHANNEL 12    CHAN12; OUTPUT CHANNEL; BITS USED TO DRIVE NAVIGATION AND SPAECRAFT HARDWARE
            //
            // #               BIT 1           ZERO RR CDU; CDU'S GIVE RRADAR INFORMATION FOR LM
            // #               BIT 2           ENABLE CDU RADAR ERROR COUNTERS
            // #               BIT 3           NOT USED
            // #               BIT 4           COARSE ALIGN ENABLE OF IMU
            // #               BIT 5           ZERO IMU CDU'S
            // #               BIT 6           ENABLE IMU ERROR COUNTER, CDU ERROR COUNTER.
            // #               BIT 7           SPARE
            // #               BIT 8           DISPLAY INERTIAL DATA
            // #               BIT 9           -PITCH GIMBAL TRIM (BELL MOTION) DESCENT ENGINE
            // #               BIT 10          +PITCH GIMBAL TRIM (BELL MOTION) DESCENT ENGINE
            // #               BIT 11          -ROLL GIMBAL TRIM (BELL MOTION) DESCENT ENGINE
            // #               BIT 12          +ROLL GIMBAL TRIM (BELL MOTION) DESCENT ENGINE
            // #               BIT 13          LR POSITION 2 COMMAND
            // #               BIT 14          ENABLE RENDESVOUS RADAR LOCK-ON;AUTO ANGLE TRACK'G
            // #               BIT 15          ISS TURN ON DELAY COMPLETE
            io::CHANNEL_CHAN12 => self.io_mem[io::CHANNEL_CHAN12],

            // ## Page 56
            // # CHANNEL 13    CHAN13; OUTPUT CHANNEL
            //
            // #               BIT 1           RADAR C         PROPER SETTING OF THE A,B,C MATRIX
            // #               BIT 2           RADAR B         SELECTS CERTAIN RADAR
            // #               BIT 3           RADAR A         PARAMETERS TO BE READ.
            // #               BIT 4           RADAR ACTIVITY
            // #               BIT 5           NOT USED (CONNECTS AN ALTERNATE INPUT TO UPLINK)
            // #               BIT 6           SPARE
            // #               BIT 7           DOWNLINK TELEMETRY WORD ORDER CODE BIT
            // #               BIT 8           RHC COUNTER ENABLE (READ HAND CONTROLLER ANGLES)
            // #               BIT 9           START RHC READ INTO COUNTERS IF BIT 8 SET
            // #               BIT 10          TEST ALARMS, TEST DSKY LIGHTS
            // #               BIT 11          ENABLE STANDBY
            // #               BIT 12          RESET TRAP 31-A         ALWAYS APPEAR TO BE SET TO 0
            // #               BIT 13          RESET TRAP 31-B         ALWAYS APPEAR TO BE SET TO 0
            // #               BIT 14          RESET TRAP 32           ALWAYS APPEAR TO BE SET TO 0
            // #               BIT 15          ENABLE T6 RUPT
            io::CHANNEL_CHAN13 => self.io_mem[io::CHANNEL_CHAN13] & 0x47CF,

            // # CHANNEL 14    CHAN14; OUTPUT CHANNEL; USED TO CONTROL COMPUTER COUNTER CELLS (CDU,GYRO,SPACECRAFT FUNC.
            //
            // #               BIT 1           OUTLINK ACTIVITY (NOT USED)
            // #               BIT 2           ALTITUDE RATE OR ALTITIDE SELECTOR
            // #               BIT 3           ALTITUDE METER ACTIVITY
            // #               BIT 4           THRUST DRIVE ACTIVITY FOR DESCENT ENGINE
            // #               BIT 5           SPARE
            // #               BIT 6           GYRO ENABLE POWER FOR PULSES
            // #               BIT 7           GYRO SELECT B           PAIR OF BITS IDENTIFIES AXIS OF -
            // #               BIT 8           GYRO SELECT A           GYRO SYSTEM TO BE TORQUED.
            // #               BIT 9           GYRO TORQUING COMMAND IN NEGATIVE DIRECTION
            // ## Page 57
            // #               BIT 10          GYRO ACTIVITY
            // #               BIT 11          DRIVE CDU S
            // #               BIT 12          DRIVE CDU T
            // #               BIT 13          DRIVE CDU Z
            // #               BIT 14          DRIVE CDU Y
            // #               BIT 15          DRIVE CDU X
            io::CHANNEL_CHAN14 => self.io_mem[io::CHANNEL_CHAN14],

            // # CHANNEL 15    MNKEYIN; INPUT CHANNEL;KEY CODE INPUT FROM KEYBOARD OF DSKY, SENSED BY PROGRAM WHEN
            // #               PROGRAM INTERRUPT #5 IS RECEIVED. USES BITS 5-1
            io::CHANNEL_MNKEYIN => {
                match &self.dsky {
                    Option::Some(x) => {
                        x.read(channel_idx)
                    },
                    Option::None => { 0o00000 }
                }
            },
            // # CHANNEL 16    NAVKEYIN; INPUT CHANNEL; OPTICS MARK INFORMATION AND NAVIGA ION PANEL DSKY (CM) OR THRUST
            // #               CONTROL (LM) SENSED BY PROGRAM WHEN PROGRAM INTER-RUPT #6 IS RECEIVED. USES BITS 3-7 ONLY.
            //
            // #               BIT 1           NOT ASSIGNED
            // #               BIT 2           NOT ASSIGNED
            // #               BIT 3           OPTICS X-AXIS MARK SIGNAL FOR ALIGN OPTICAL TSCOPE
            // #               BIT 4           OPTICS Y-AXIS MARK SIGNAL FOR AOT
            // #               BIT 5           OPTICS MARK REJECT SIGNAL
            // #               BIT 6           DESCENT+ ; CREW DESIRED SLOWING RATE OF DESCENT
            // #               BIT 7           DESCENT- ; CREW DESIRED SPEEDING UP RATE OF D'CENT
            io::CHANNEL_NAVKEYIN => 0,

            // # NOTE: ALL BITS IN CHANNELS 30-33 ARE INVERTED AS SENSED BY THE  PROGRAM, SO THAT A VALUE OF ZERO MEANS
            // # THAT THE INDICATED SIGNAL IS PRESENT.
            //
            // # CHANNEL 30    INPUT CHANNEL
            //
            // #               BIT 1           ABORT WITH DESCENT STAGE
            // #               BIT 2              UNUSED
            // #               BIT 3           ENGINE ARMED SIGNAL
            // #               BIT 4           ABORT WITH ASCENT ENGINE STAGE
            // #               BIT 5           AUTO THROTTLE; COMPUTER CONTROL OF DESCENT ENGINE
            // ## Page 58
            // #               BIT 6           DISPLAY INERTIAL DATA
            // #               BIT 7           RR CDU FAIL
            // #               BIT 8           SPARE
            // #               BIT 9           IMU OPERATE WITH NO MALFUNCTION
            // #               BIT 10          LM COMPUTER (NOT AGS) HAS CONTROL OF LM
            // #               BIT 11          IMU CAGE COMMAND TO DRIVE IMU GIMBAL ANGLES TO 0.
            // #               BIT 12          IMU CDU FAIL (MALFUNCTION OF IMU CDU,S)
            // #               BIT 13          IMU FAIL (MALFUNCTION OF IMU STABILIZATION LOOPS)
            // #               BIT 14          ISS TURN ON REQUESTED
            // #               BIT 15          TEMPERATURE OF STABLE MEMBER WITHIN DESIGN LIMITS
            io::CHANNEL_CHAN30 => self.handle_channel30_read(),

            // # CHANNEL 31    INPUT CHANNEL; BITS ASSOCIATED WITH THE ATTITUDE CONTROLLER, TRANSLATIONAL CONTROLLER,
            // #               AND SPACECRAFT ATTITUDE CONTROL; USED BY RCS DAP
            //
            // #               BIT 1           ROTATION (BY RHC) COMMANDED IN POSITIVE PITCH DIRECTION; MUST BE IN MINIMUM IMPULSE MODE.
            // #                               ALSO POSITIVE ELEVATION CHANGE FOR LANDING POINT  DESIGNATOR
            // #               BIT 2           AS BIT 1 EXCEPT NEGATIVE PITCH AND ELEVATION
            // #               BIT 3           ROTATION (BY RHC) COMMANDED IN POSITIVE YAW DIRECTION; MUST BE IN MINUMUM IMPULSE MODE.
            // #               BIT 4           AS BIT 3 EXCEPT NEGATIVE YAW
            // #               BIT 5           ROTATION (BY RHC) COMMANDED IN POSITIVE ROLL DIRECTION; MUST BE IN MINIMUM IMPULSE MODE.
            // #                               ALSO POSITIVE AZIMUTH CHANGE FOR LANDING POINT DESIGNATOR
            // #               BIT 6           AS BIT 5 EXCEPT NEGATIVE ROLL AND AZIMUTH
            // #               BIT 7           TRANSLATION IN +X DIRECTION COMMANDED BY THC
            // #               BIT 8           TRANSLATION IN -X DIRECTION COMMANDED BY THC
            // #               BIT 9           TRANSLATION IN +Y DIRECTION COMMANDED BY THC
            // #               BIT 10          TRANSLATION IN -Y DIRECTION COMMANDED BY THC
            // #               BIT 11          TRANSLATION IN +Z DIRECTION COMMANDED BY THC
            // #               BIT 12          TRANSLATION IN -Z DIRECTION COMMANDED BY THC
            // ## Page 59
            // #               BIT 13          ATTITUDE HOLD MODE ON SCS MODE CONTROL SWITCH
            // #               BIT 14          AUTO STABILIZATION OF ATTITUDE ON SCS MODE SWITCH
            // #               BIT 15          ATTITUDE CONTROL OUT OF DETENT (RHC NOT IN NEUTRAL
            io::CHANNEL_CHAN31 => 0o77777,

            // # CHANNEL 32    INPUT CHANNEL.
            //
            // #               BIT 1              THRUSTERS 2 & 4 DISABLED BY CREW
            // #               BIT 2              THRUSTERS 5 & 8 DISABLED BY CREW
            // #               BIT 3              THRUSTERS 1 & 3 DISABLED BY CREW
            // #               BIT 4              THRUSTERS 6 & 7 DISABLED BY CREW
            // #               BIT 5              THRUSTERS 14 & 16 DISABLED BY CREW
            // #               BIT 6              THRUSTERS 13 & 15 DISABLED BY CREW
            // #               BIT 7              THRUSTERS 9 & 12 DISABLED BY CREW
            // #               BIT 8              THRUSTERS 10 & 11 DISABLED BY CREW
            // #               BIT 9              DESCENT ENGINE GIMBALS DISABLED BY CREW
            // #               BIT 10             APPARENT DESCENT ENGINE GIMBAL FAILURE
            // #               BIT 14             INDICATES PROCEED KEY IS DEPRESSED
            io::CHANNEL_CHAN32 => {
                let val = match &self.dsky {
                    Option::Some(x) => {
                        x.read(channel_idx)
                    },
                    Option::None => { 0o77777 }
                };
                //println!("CHAN32: {:5o}", val);
                val | (self.io_mem[0o32] & 0o57777)
            }

            // # CHANNEL 33    CHAN33; INPUT CHANNEL; FOR HARDWARE STATUS AND COMMAND INFORMATION. BITS 15-11 ARE FLIP-
            // #               FLOP BITS RESET BY A CHANNEL "WRITE" COMMAND THAT ARE RESET BY A RESTART & BY T4RUPT LOOP.
            //
            // #               BIT 1           SPARE
            // #               BIT 2           RR AUTO-POWER ON
            // #               BIT 3           RR RANGE LOW SCALE
            // #               BIT 4           RR DATA GOOD
            // #               BIT 5           LR RANGE DATA GOOD
            // #               BIT 6           LR POS1
            // #               BIT 7           LR POS2
            // ## Page 60
            // #               BIT 8           LR VEL DATA GOOD
            // #               BIT 9           LR RANGE LOW SCALE
            // #               BIT 10          BLOCK UPLINK INPUT
            // #               BIT 11          UPLINK TOO FAST
            // #               BIT 12          DOWNLINK TOO FAST
            // #               BIT 13          PIPA FAIL
            // #               BIT 14          WARNING OF REPEATED ALARMS: RESTART,COUNTER FAIL, VOLTAGE FAIL,AND SCALAR DOUBLE.
            // #               BIT 15          LGC OSCILLATOR STOPPED
            io::CHANNEL_CHAN33 => 0o77777,

            // # CHANNEL 34    DNT M1; OUTPUT CHANNEL; DOWNLINK 1  FIRST OF TWO WORDS SERIALIZATION.
            // # CHANNEL 35    DNT M2; OUTPUT CHANNEL DOWNLINK 2 SOCOND OF TWO   WORDS SERIALIZATION.
            io::CHANNEL_CHAN34 | io::CHANNEL_CHAN35 => {
                match &self.downrupt {
                    Option::Some(x) => {
                        x.read(channel_idx)
                    },
                    Option::None => { 0o77777 }
                }
            },
            0o163 => {
                match &self.dsky {
                    Option::Some(x) => {
                        x.read(channel_idx)
                    },
                    Option::None => { 0o77777 }
                }
            },
            _ => {
                //error!("Unknown IO Channel: {:o}", channel_idx);
                self.io_mem[channel_idx]
            }
        }
    }

    pub fn write(&mut self, channel_idx: usize, val: u16) {
        //debug!("IO Space Write: {:x} {:x}", channel_idx, val);

        // Handle the case of the DSKY Object and perform a write,
        // if needed
        match &mut self.dsky {
            Option::Some(x) => {
                x.write(channel_idx, val);
            }
            _ => {}
        }

        // Handle the case of the downrupt Object and perform a write,
        // if needed
        match &mut self.downrupt {
            Option::Some(x) => {
                x.write(channel_idx, val);
            }
            _ => {}
        }

        match channel_idx {
            io::CHANNEL_DSALMOUT => {
                self.io_mem[io::CHANNEL_DSALMOUT] = val; //val & 0x33FF;
            }
            io::CHANNEL_SUPERBNK => self.io_mem[channel_idx] = val & 0o00160,
            io::CHANNEL_CHAN13 => {
                self.io_mem[io::CHANNEL_CHAN13] = val;
            }
            io::CHANNEL_CHAN32 => {
                //warn!("Attempting to write to IO CHAN32 which is only an input");
            }
            _ => {
                self.io_mem[channel_idx] = val;
            }
        }
    }

    pub fn check_interrupt(&mut self) -> u16 {
        let mut val = 0;

        val |= match &mut self.dsky {
            Option::Some(x) => {
                x.is_interrupt()
            }
            Option::None => { 0o00000 },
        };

        val |= match &mut self.downrupt {
            Option::Some(x) => {
                x.is_interrupt()
            }
            Option::None => { 0o00000 },
        };

        val
    }
}
