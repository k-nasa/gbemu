#![feature(exclusive_range_pattern)]

//! Game Boy emulator core implementation.

use std::sync::{Arc, Mutex};

pub mod bus;
pub mod cartridge;
pub(crate) mod cpu;
pub mod emulator;
pub mod gpu;
pub(crate) mod logger;
pub mod ram;

pub(crate) type Word = u16;
pub(crate) type HalfWord = u8;
pub type ShareBus = Arc<Mutex<bus::Bus>>;
pub type SharedGpu = Arc<Mutex<gpu::Gpu>>;

pub(crate) fn join_half_words(upper: HalfWord, lower: HalfWord) -> Word {
    (upper as u16) << 8 ^ lower as u16
}

pub(crate) fn split_word(word: Word) -> (HalfWord, HalfWord) {
    ((word >> 8) as HalfWord, (word & 0x00FF) as HalfWord)
}
