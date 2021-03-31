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

    pub fn read(&mut self, target: TargetRegister) -> HalfWord {
        match target {
            TargetRegister::A => self.a,
            TargetRegister::B => self.b,
            TargetRegister::C => self.c,
            TargetRegister::D => self.d,
            TargetRegister::E => self.e,
            TargetRegister::F => self.f,
            TargetRegister::H => self.h,
            TargetRegister::L => self.l,
        }
    }
}

#[derive(Clone, Copy)]
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
                // LD BC, u16
                let operands = self.fetch_operands(2);
                self.ldnn_u16(TargetRegister::B, TargetRegister::C, operands)
            }
            0x02 => {
                // LD (BC),A
                self.ldrr_r(TargetRegister::B, TargetRegister::C, TargetRegister::A);
            }
            0x03 => {
                // INC BC
                self.inc_u16(TargetRegister::B, TargetRegister::C);
            }
            0x04 => {
                // INC B
                self.inc_u8(TargetRegister::B);
            }
            0x05 => {}
            0x06 => {}
            0x07 => {}
            0x08 => {}
            0x09 => {}
            0x0A => {}
            _ => bail!("not implemented opcode {:X}", opcode),
        }
        // {0x4, "INC B", 0, 1,  cpu.inc_n(&cpu.Regs.B) }},
        // {0x5, "DEC B", 0, 1,  cpu.dec_n(&cpu.Regs.B) }},
        // {0x6, "LD B,n", 1, 2, cpu.ldnn_n(&cpu.Regs.B, operands) }},

        Ok(())
    }

    fn ldnn_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister, ops: Operands) {
        self.registers.write(reg1, ops[1]);
        self.registers.write(reg2, ops[0]);
    }

    fn ldrr_r(
        &mut self,
        upper_reg: TargetRegister,
        lower_reg: TargetRegister,
        byte_reg: TargetRegister,
    ) {
        let address = join_half_words(
            self.registers.read(upper_reg),
            self.registers.read(lower_reg),
        );

        let byte = self.registers.read(byte_reg);
        self.bus.write_byte(address, byte);
    }

    fn inc_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister) {
        let mut word = join_half_words(self.registers.read(reg1), self.registers.read(reg2));
        word += 1;

        let (upper, lower) = split_word(word);

        self.registers.write(reg1, upper);
        self.registers.write(reg2, lower);
    }

    fn inc_u8(&mut self, reg: TargetRegister) {
        todo!()
    }
}

fn join_half_words(upper: HalfWord, lower: HalfWord) -> Word {
    (upper as u16) << 8 ^ lower as u16
}

fn split_word(word: Word) -> (HalfWord, HalfWord) {
    ((word >> 8) as HalfWord, (word ^ 0x00FF) as HalfWord)
}
