type Word = u16;
type HalfWord = u8;

#[derive(Debug)]
struct Registers {
    a: HalfWord,
    f: HalfWord,
    b: HalfWord,
    c: HalfWord,
    d: HalfWord,
    e: HalfWord,
    h: HalfWord,
    l: HalfWord,
}

pub struct Cpu {
    registers: Registers,
    pc: Word,
    sp: Word,
}

impl Cpu {
    pub fn new() -> Cpu {
        todo!()
    }

    pub fn step() -> Result<(), ()> {
        todo!()
    }

    fn fetch() -> Instruction {
        todo!()
    }
}

struct Instruction {
    opecode: u8,
    description: String,
    length_in_bytes: u8,
    duration_in_cycle: u8,
}
