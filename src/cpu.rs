use crate::{bus::Bus, HalfWord, Word};
use anyhow::{bail, Result};

type Opecode = u8;
type Operands = Vec<u8>;

#[derive(Debug)]
struct Registers {
    pub a: HalfWord,
    pub f: HalfWord,
    pub b: HalfWord,
    pub c: HalfWord,
    pub d: HalfWord,
    pub e: HalfWord,
    pub h: HalfWord,
    pub l: HalfWord,
}

impl Registers {
    pub fn write(&mut self, target: TargetRegister, half_word: HalfWord) {
        match target {
            TargetRegister::A => self.a = half_word,
            TargetRegister::B => self.b = half_word,
            TargetRegister::C => self.c = half_word,
            TargetRegister::D => self.d = half_word,
            TargetRegister::E => self.e = half_word,
            TargetRegister::F => self.f = half_word,
            TargetRegister::H => self.h = half_word,
            TargetRegister::L => self.l = half_word,
        }
    }
}

enum TargetRegister {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

// ref http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
const INIT_PC: Word = 0x100;
const INIT_SP: Word = 0xFFFE;

pub struct Cpu {
    registers: Registers,
    pc: Word,
    sp: Word,
    bus: Bus,
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        Cpu {
            pc: INIT_PC,
            sp: INIT_SP,
            registers: Registers {
                a: 0x11,
                f: 0x80,
                b: 0x00,
                c: 0x00,
                d: 0xFF,
                e: 0x56,
                h: 0x00,
                l: 0x0D,
            },
            bus,
        }
    }

    pub fn step(&mut self) -> Result<()> {
        let opcode = self.fetch();

        self.execute(opcode)?;

        Ok(())
    }

    fn fetch(&mut self) -> Opecode {
        let opcode = self.bus.read_byte(self.pc);
        self.pc += 1;

        opcode
    }

    fn fetch_operands(&mut self, length_in_bytes: usize) -> Vec<u8> {
        (0..length_in_bytes).map(|_| self.fetch()).collect()
    }

    // opcode list https://izik1.github.io/gbops/
    fn execute(&mut self, opcode: Opecode) -> Result<()> {
        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                let operands = self.fetch_operands(2);
                self.ldnn_u16(TargetRegister::B, TargetRegister::C, operands)
            }
            _ => bail!("not implemented opcode {:X}", opcode),
        }

        Ok(())
    }

    fn ldnn_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister, ops: Operands) {
        self.registers.write(reg1, ops[1]);
        self.registers.write(reg2, ops[0]);
    }
}
