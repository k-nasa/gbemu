use crate::{HalfWord, Word};

pub struct Gpu {
    // FIXME ダミー実装
    data: Vec<u8>,
}

impl Gpu {
    pub fn with_size(size: usize) -> Gpu {
        Gpu {
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
