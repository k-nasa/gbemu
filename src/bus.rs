use crate::cartridge::Cartridge;
use crate::gpu::Gpu;
use crate::ram::Ram;
use crate::SharedGpu;
use crate::{split_word, HalfWord, Word};

/// Memory map
/// Ref http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
/// Ref https://w.atwiki.jp/gbspec/pages/13.html
///```
/// Interrupt Enable Register
/// --------------------------- FFFF
/// Internal RAM
/// --------------------------- FF80
/// Empty but unusable for I/O
/// --------------------------- FF4C
/// I/O ports
/// --------------------------- FF00
/// Empty but unusable for I/O
/// --------------------------- FEA0
/// Sprite Attrib Memory (OAM)
/// --------------------------- FE00
/// Echo of 8kB Internal RAM
/// --------------------------- E000
/// 8kB Internal RAM
/// --------------------------- C000
/// 8kB switchable RAM bank
/// --------------------------- A000
/// 8kB Video RAM
/// --------------------------- 8000 --
/// 16kB switchable ROM bank |
/// --------------------------- 4000 |= 32kB Cartrigbe
/// 16kB ROM bank #0 |
/// --------------------------- 0000 --
/// ```
pub struct Bus {
    h_ram: Ram,
    oam_ram: Ram,
    mirror_ram: Ram,
    working_ram: Ram,
    video_ram: Ram,
    cartridge: Cartridge,
    gpu: SharedGpu,
}

impl Bus {
    pub fn new(
        cartridge: Cartridge,
        video_ram: Ram,
        h_ram: Ram,
        oam_ram: Ram,
        mirror_ram: Ram,
        working_ram: Ram,
        gpu: SharedGpu,
    ) -> Bus {
        Bus {
            h_ram,
            oam_ram,
            working_ram,
            mirror_ram,
            cartridge,
            video_ram,
            gpu,
        }
    }

    pub fn read_byte(&self, address: Word) -> u8 {
        let device = Device::resolve_bus_address(address);

        match device {
            Device::HRam(address) => self.h_ram.read(address),
            Device::OamRam(address) => self.oam_ram.read(address),
            Device::MirrorRam(address) => self.mirror_ram.read(address),
            Device::WorkingRam(address) => self.working_ram.read(address),
            Device::VideoRam(address) => self.video_ram.read(address),
            Device::Cartridge(address) => self.cartridge.read(address),
            Device::Gpu(address) => self.gpu.lock().unwrap().read(address),
            Device::Timer(_) => todo!(),
            Device::P1 => todo!(),
            Device::DIV => todo!(),
            Device::IF => todo!(),
            Device::Unimplement => 0,
        }
    }

    pub fn write_byte(&mut self, address: Word, byte: HalfWord) {
        let device = Device::resolve_bus_address(address);

        match device {
            Device::HRam(address) => self.h_ram.write(address, byte),
            Device::OamRam(address) => self.oam_ram.write(address, byte),
            Device::MirrorRam(address) => self.mirror_ram.write(address, byte),
            Device::WorkingRam(address) => self.working_ram.write(address, byte),
            Device::VideoRam(address) => self.video_ram.write(address, byte),
            Device::Cartridge(address) => self.cartridge.write(address, byte),
            Device::Gpu(address) => self.gpu.lock().unwrap().write(address, byte),
            Device::Timer(_) => todo!(),
            Device::P1 => todo!(),
            Device::DIV => todo!(),
            Device::IF => todo!(),
            Device::Unimplement => log::warn!("unimplemented addr {}", address),
        }
    }

    pub fn write_word(&mut self, address: Word, word: Word) {
        let (upper, lower) = split_word(word);

        self.write_byte(address, lower);
        self.write_byte(address + 1, upper);
    }
}

type Address = Word;
#[derive(Debug)]
enum Device {
    HRam(Address),
    OamRam(Address),
    MirrorRam(Address),
    WorkingRam(Address),
    VideoRam(Address),
    Cartridge(Address),
    Gpu(Address),
    P1,
    IF,
    DIV,
    Timer(Address),
    Unimplement,
}

impl Device {
    pub fn resolve_bus_address(addr: Word) -> Device {
        match addr {
            0x0000..0x8000 => Device::Cartridge(addr),
            0x8000..0xA000 => Device::VideoRam(addr - 0x8000),
            0xA000..0xC000 => Device::Cartridge(addr),
            0xC000..0xE000 => Device::WorkingRam(addr - 0xC000),
            0xE000..0xFE00 => Device::MirrorRam(addr - 0xE000),
            0xFE00..0xFEA0 => Device::OamRam(addr - 0xFE00),
            0xFF80..=0xFFFF => Device::HRam(addr - 0xFF80),
            0xFF40..0xFF80 => Device::Gpu(addr - 0xFF40),
            0xFF00 => {
                // TODO Padの実装が入る
                log::warn!("TODO: implement Pad device");
                Device::Unimplement
            }
            0xFF04 => {
                // TODO DIV の実装が入る
                log::warn!("TODO: implement DIV register");
                Device::Unimplement
            }
            0xFF05..0xFF08 => {
                // TODO Timerの実装が入る
                log::warn!("TODO: implement timer");
                Device::Unimplement
            }
            0xFF0F => {
                // TODO IF の実装が入る
                log::warn!("TODO: implement IF device");
                Device::Unimplement
            }
            _ => Device::Unimplement,
        }
    }
}
