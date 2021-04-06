use crate::SharedBus;
use crate::{HalfWord, Word};
use std::cell::RefCell;

pub struct Gpu {
    // FIXME ダミー実装
    data: Vec<u8>,
    bus: Option<RefCell<SharedBus>>,
}

impl Gpu {
    pub fn new(size: usize, bus: Option<RefCell<SharedBus>>) -> Gpu {
        Gpu {
            data: vec![0; size],
            bus,
        }
    }

    pub fn step(&mut self) {}

    pub fn set_bus(&mut self, bus: RefCell<SharedBus>) {
        self.bus = Some(bus)
    }

    pub fn read(&self, address: Word) -> HalfWord {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, byte: HalfWord) {
        self.data[address as usize] = byte
    }
}
