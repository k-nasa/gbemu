use crate::cpu::Cpu;

pub struct Emulator {
    cpu: Cpu,
}

impl Emulator {
    pub fn new() -> Emulator {
        todo!()
    }

    pub fn start(mut self) {
        loop {
            // TODO error handle
            let _ = self.cpu.step();
        }
    }
}
