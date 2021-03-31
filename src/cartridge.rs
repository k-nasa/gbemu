use crate::Word;

pub struct Cartridge {
    data: Vec<u8>,
}

impl Cartridge {
    pub fn read(&self, address: Word) -> u8 {
        self.data[address as usize]
    }
}
