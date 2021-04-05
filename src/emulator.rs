use crate::cpu::Cpu;
use crate::logger::{Logger, LoggerImpl};
use crate::{SharedBus, SharedGpu};
use anyhow::Result;

pub struct Emulator {
    cpu: Cpu<dyn Logger>,
    gpu: SharedGpu,
}

impl Emulator {
    pub fn new(bus: SharedBus, gpu: SharedGpu) -> Self {
        Emulator {
            cpu: Cpu::new(Box::new(LoggerImpl {}), bus),
            gpu,
        }
    }

    pub fn start(mut self) -> Result<()> {
        loop {
            self.cpu.step()?;

            let mut gpu = self.gpu.lock().unwrap();
            gpu.step();
        }
    }
}
