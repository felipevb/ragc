

/* Number of Erasable Banks within a given AGC computer */
pub const RAM_NUM_BANKS: usize = 8;

/* Number of words within a given Erasable memory bank */
pub const RAM_BANK_NUM_WORDS: usize = 256;

/* Number of Fixed Banks within a given AGC computer */
pub const ROM_NUM_BANKS: usize = 36;

/* Number of words within a given FIXED memory bank */
pub const ROM_BANK_NUM_WORDS: usize = 1024;


pub mod io {
    pub const CHANNEL_L: usize = 0o01;
    pub const CHANNEL_Q: usize = 0o02;
    pub const CHANNEL_HISCALAR: usize = 0o03;
    pub const CHANNEL_LOSCALAR: usize = 0o04;
    pub const CHANNEL_PYJETS: usize = 0o05;
    pub const CHANNEL_ROLLJETS: usize = 0o06;
    pub const CHANNEL_SUPERBNK: usize = 0o07; // Only bits[7:5] and only 0XX or 100 are
                                              // valid
    pub const CHANNEL_DSKY: usize = 0o10;
    pub const CHANNEL_DSALMOUT: usize = 0o11;
    pub const CHANNEL_CHAN12: usize = 0o12;
    pub const CHANNEL_CHAN13: usize = 0o13;
    pub const CHANNEL_CHAN14: usize = 0o14;
    pub const CHANNEL_MNKEYIN: usize = 0o15;
    pub const CHANNEL_NAVKEYIN: usize = 0o16;

    pub const CHANNEL_CHAN30: usize = 0o30;
    pub const CHANNEL_CHAN31: usize = 0o31;
    pub const CHANNEL_CHAN32: usize = 0o32;
    pub const CHANNEL_CHAN33: usize = 0o33;
    pub const CHANNEL_CHAN34: usize = 0o34; // DOWNLIST WORD1
    pub const CHANNEL_CHAN35: usize = 0o35; // DOWNLIST WORD2
}

pub mod cpu {
    pub const REG_A: usize = 0x0;
    pub const REG_L: usize = 0x1; // Original Name
    pub const REG_B: usize = 0x1;
    pub const REG_Q: usize = 0x02; // Original Name
    pub const REG_LR: usize = 0x2;
    pub const REG_EB: usize = 0x3;
    pub const REG_FB: usize = 0x4;
    pub const REG_Z: usize = 0x05;
    pub const REG_PC: usize = 0x05;
    pub const REG_BB: usize = 0x6;
    pub const REG_ZERO: usize = 0x7;
    pub const REG_A_SHADOW: usize = 0x8;
    pub const REG_B_SHADOW: usize = 0x9;
    pub const REG_LR_SHADOW: usize = 0xA;
    pub const REG_EB_SHADOW: usize = 0xB;
    pub const REG_FB_SHADOW: usize = 0xC;
    pub const REG_PC_SHADOW: usize = 0xD;
    pub const REG_BB_SHADOW: usize = 0xE;

    pub const REG_IR: usize = 0xF;
    pub const REG_MAX: usize = 0x10;

    pub const RUPT_RESET: u8 = 0x0;
    pub const RUPT_TIME6: u8 = 0x1;
    pub const RUPT_TIME5: u8 = 0x2;
    pub const RUPT_TIME3: u8 = 0x3;
    pub const RUPT_TIME4: u8 = 0x4;
    pub const RUPT_KEY1: u8 = 0x5;
    pub const RUPT_KEY2: u8 = 0x6;
    pub const RUPT_UPRUPT: u8 = 0x7;
    pub const RUPT_DOWNRUPT: u8 = 0x8;
    pub const RUPT_RADAR: u8 = 0x9;
    pub const RUPT_HANDRUPT: u8 = 0xA;

    pub const NIGHTWATCH_TIME: u32 = 1920000000 / 11700;

    // Each TC/TCF is 1 cycle, so we just need to have to know how many cycles it
    // takes for 15ms and thats how many TC/TCF instructions we have to see in
    // sequence to reset.
    pub const TCMONITOR_COUNT: u32 = 15000000 / 11700;

    pub const RUPT_LOCK_COUNT: i32 = 300000000 / 11700;
}

pub mod edit {
    pub const SG_CYR: usize = 0o20;
    pub const SG_SR: usize = 0o21;
    pub const SG_CYL: usize = 0o22;
    pub const SG_EDOP: usize = 0o23;
}

pub mod timer {
    pub const MM_TIME2: usize = 0o24;
    pub const MM_TIME1: usize = 0o25;
    pub const MM_TIME3: usize = 0o26;
    pub const MM_TIME4: usize = 0o27;
    pub const MM_TIME5: usize = 0o30;
    pub const MM_TIME6: usize = 0o31;
}

pub mod special {
    pub const SG_CDUX: usize = 0o32;
    pub const SG_CDUY: usize = 0o33;
    pub const SG_CDUZ: usize = 0o34;
    pub const SG_OPTY: usize = 0o35;
    pub const SG_OPTX: usize = 0o36;
    pub const SG_PIPAX: usize = 0o37;
    pub const SG_PIPAY: usize = 0o40;
    pub const SG_PIPAZ: usize = 0o41;
    pub const SG_RCHP: usize = 0o42;
    pub const SG_RCHY: usize = 0o43;
    pub const SG_RCHR: usize = 0o44;
    pub const SG_INLINK: usize = 0o45;
    pub const SG_RNRAD: usize = 0o46;
    pub const SG_GYROCTR: usize = 0o47;
    pub const SG_CDUXCMD: usize = 0o50;
    pub const SG_CDUYCMD: usize = 0o51;
    pub const SG_CDUZCMD: usize = 0o52;
    pub const SG_OPTYCMD: usize = 0o53;
    pub const SG_OPTXCMD: usize = 0o54;
    pub const SG_THRUST: usize = 0o55; // LM only
    pub const SG_LEMONM: usize = 0o56; // LM only
    pub const SG_OUTLINK: usize = 0o57;
    pub const SG_ALTM: usize = 0o60; // LM Only
}