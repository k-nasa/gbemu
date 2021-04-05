use crate::ShareBus;
use crate::{HalfWord, Word};

pub struct Gpu {
    // FIXME ダミー実装
    data: Vec<u8>,
    bus: Option<ShareBus>,
}

impl Gpu {
    pub fn new(size: usize, bus: Option<ShareBus>) -> Gpu {
        Gpu {
            data: vec![0; size],
            bus,
        }
    }

    pub fn set_bus(&mut self, bus: ShareBus) {
        self.bus = Some(bus);
    }

    pub fn read(&self, address: Word) -> HalfWord {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, byte: HalfWord) {
        self.data[address as usize] = byte
    }
}
