use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gpu::Gpu;
use crate::logger::{Logger, LoggerImpl};
use crate::ram::Ram;
use crate::{SharedBus, SharedGpu};
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct Emulator {
    cpu: Cpu,
    gpu: SharedGpu,
}

impl Emulator {
    pub fn new(bus: SharedBus, gpu: SharedGpu) -> Self {
        Emulator {
            cpu: Cpu::new(bus),
            gpu,
        }
    }

    pub fn from_rom_byte(bytes: Vec<u8>) -> Emulator {
        // NOTE https://w.atwiki.jp/gbspec/pages/13.html サイズはこれを見て決めた
        let video_ram = Ram::with_size(0x2000);
        let h_ram = Ram::with_size(0x2000);
        let oam_ram = Ram::with_size(0x2000);
        let mirror_ram = Ram::with_size(0x2000);
        let working_ram = Ram::with_size(0x2000);
        let cartridge = Cartridge::new(bytes);
        let gpu = Gpu::new(1024, None); // TODO implement
        let gpu = Arc::new(Mutex::new(gpu));

        let bus = Bus::new(
            cartridge,
            video_ram,
            h_ram,
            oam_ram,
            mirror_ram,
            working_ram,
            gpu.clone(),
        );

        let bus = Arc::new(Mutex::new(bus));
        gpu.lock().unwrap().set_bus(bus.clone());

        Emulator::new(bus.clone(), gpu.clone())
    }

    pub fn start(mut self) -> Result<()> {
        loop {
            self.cpu.step()?;

            let mut gpu = self.gpu.lock().unwrap();
            gpu.step();
        }
    }
}
