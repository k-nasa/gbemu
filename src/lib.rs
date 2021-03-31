#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]

//! Game Boy emulator core implementation.

pub(crate) mod bus;
pub(crate) mod cartridge;
pub(crate) mod cpu;
pub mod emulator;

type Word = u16;
type HalfWord = u8;
