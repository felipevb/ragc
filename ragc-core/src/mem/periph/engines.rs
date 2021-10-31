use super::Peripheral;

pub const LM_DECEND_ENGINE: u8 = 0;
#[allow(dead_code)]
pub const LM_ASCEND_ENGINE: u8 = 1;
#[allow(dead_code)]
pub const LM_UNKNOWN_ENGINE: u8 = 2;

// =============================================================================
//  Generic Engine Template
// =============================================================================

trait Engine {
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enable: bool);
}

// =============================================================================
//  LM Decend Engine
// =============================================================================
pub struct LmDecendEngine {
    enabled: bool,
}

impl Engine for LmDecendEngine {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enable: bool) {
        self.enabled = enable;
    }
}

impl LmDecendEngine {
    pub fn new() -> LmDecendEngine {
        LmDecendEngine { enabled: false }
    }
}

// =============================================================================
//  Peripherial Interface for all LM engines
// =============================================================================
pub struct LmEngines {
    decend_engine: LmDecendEngine,
    active: u8,
}

impl Peripheral for LmEngines {
    fn is_interrupt(&mut self) -> u16 {
        0
    }
}

impl LmEngines {
    pub fn new() -> LmEngines {
        LmEngines {
            active: LM_DECEND_ENGINE,
            decend_engine: LmDecendEngine::new(),
        }
    }

    pub fn _get_engine_enable(&self) -> bool {
        match self.active {
            LM_DECEND_ENGINE => self.decend_engine.is_enabled(),
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn set_engine_enable(&mut self, enable: bool) {
        match self.active {
            LM_DECEND_ENGINE => self.decend_engine.set_enabled(enable),
            _ => {}
        }
    }

    pub fn get_active_engine(&self) -> u8 {
        self.active
    }

    #[allow(dead_code)]
    pub fn set_active_engine(&mut self, engine_type: u8) {
        if engine_type < LM_UNKNOWN_ENGINE {
            self.active = engine_type;
        }
    }
}
