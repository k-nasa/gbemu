use crate::{HalfWord, Word};

pub struct Cartridge {
    pub data: Vec<u8>,
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Cartridge {
        Cartridge { data }
    }

    pub fn read(&self, address: Word) -> u8 {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, byte: HalfWord) {
        self.data[address as usize] = byte
    }
}
