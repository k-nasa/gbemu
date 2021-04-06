use crate::SharedBus;
use crate::{HalfWord, Word};

pub struct Gpu {
    // FIXME ダミー実装
    data: Vec<u8>,
    bus: Option<SharedBus>,
}

impl Gpu {
    pub fn new(size: usize, bus: Option<SharedBus>) -> Gpu {
        Gpu {
            data: vec![0; size],
            bus,
        }
    }

    pub fn step(&mut self) {}

    pub fn set_bus(&mut self, bus: SharedBus) {
        self.bus = Some(bus)
    }

    pub fn read(&self, address: Word) -> HalfWord {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, byte: HalfWord) {
        self.data[address as usize] = byte;

        let bus = self.bus.as_ref().unwrap();
        let mut bus = bus.lock().unwrap();
        bus.write_byte(address, byte);
    }
}
