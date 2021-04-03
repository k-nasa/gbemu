use crate::cartridge::Cartridge;
use crate::{split_word, HalfWord, Word};

/// Memory map
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
pub struct Bus {
    cartridge: Cartridge,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Bus {
        Bus { cartridge }
    }

    pub fn read_byte(&self, address: Word) -> u8 {
        // FIXME アドレスとデバイスの解決をメソッドに切り出して write_byteと共通化する
        match address {
            0x000..=0x7FFF => self.cartridge.read(address),
            _ => todo!(),
        }
    }

    pub fn write_byte(&mut self, address: Word, byte: HalfWord) {
        // FIXME アドレスとデバイスの解決をメソッドに切り出して write_byteと共通化する
        match address {
            0x000..=0x7FFF => self.cartridge.write(address, byte),
            _ => todo!(),
        }
    }

    pub fn write_word(&mut self, address: Word, word: Word) {
        let (upper, lower) = split_word(word);

        self.write_byte(address, lower);
        self.write_byte(address + 1, upper);
    }
}
