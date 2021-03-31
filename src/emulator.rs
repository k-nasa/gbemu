use crate::bus::Bus;
use crate::cpu::Cpu;

pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    pub fn new(bus: Bus) -> Emulator {
        Emulator { cpu: Cpu::new(bus) }
    }

    pub fn start(mut self) {
        loop {
            // TODO error handle
            let _ = self.cpu.step();
        }
    }
}
