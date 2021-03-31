use crate::cartridge::Cartridge;
use crate::{HalfWord, Word};

pub struct Bus {
    cartridge: Cartridge,
}

impl Bus {
    /// Ref http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
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
    pub fn read_byte(&self, address: Word) -> u8 {
        match address {
            0x000..=0x7FFF => self.cartridge.read(address),
            _ => todo!(),
        }
    }
}
