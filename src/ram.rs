use crate::{HalfWord, Word};

pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn with_size(size: usize) -> Ram {
        Ram {
            data: vec![0; size],
        }
    }

    pub fn read(&self, address: Word) -> HalfWord {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, byte: HalfWord) {
        self.data[address as usize] = byte
    }
}
