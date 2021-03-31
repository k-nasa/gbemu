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
            match self.cpu.step() {
                Ok(_) => {}
                Err(e) => {
                    println!("error: {:?}", e);
                    return;
                }
            }
        }
    }
}
