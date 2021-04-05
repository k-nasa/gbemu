use crate::cpu::Cpu;
use crate::logger::{Logger, LoggerImpl};
use crate::ShareBus;
use anyhow::Result;

pub struct Emulator {
    cpu: Cpu<dyn Logger>,
}

impl Emulator {
    pub fn new(bus: ShareBus) -> Self {
        Emulator {
            cpu: Cpu::new(Box::new(LoggerImpl {}), bus),
        }
    }

    pub fn start(mut self) -> Result<()> {
        loop {
            self.cpu.step()?;
        }
    }
}
