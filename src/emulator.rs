use crate::bus::Bus;
use crate::cpu::Cpu;
use anyhow::Result;

pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    pub fn new(bus: Bus) -> Emulator {
        Emulator { cpu: Cpu::new(bus) }
    }

    pub fn start(mut self) -> Result<()> {
        loop {
            self.cpu.step()?;
        }
    }
}
