use crate::{bus::Bus, HalfWord, Word};
use anyhow::{bail, Result};

type Opecode = u8;
type Operands = Vec<u8>;

/// # Registers
///  16bit Hi   Lo   Name/Function
///  AF    A    -    Accumulator & Flags
///  BC    B    C    BC
///  DE    D    E    DE
///  HL    H    L    HL
///  SP    -    -    Stack Pointer
///  PC    -    -    Program Counter/Pointer
#[derive(Debug)]
struct Registers {
    pub a: HalfWord,
    pub b: HalfWord,
    pub c: HalfWord,
    pub d: HalfWord,
    pub e: HalfWord,
    pub h: HalfWord,
    pub l: HalfWord,
    pub f: FlagRegister,
}

impl Registers {
    pub fn write(&mut self, target: TargetRegister, half_word: HalfWord) {
        match target {
            TargetRegister::A => self.a = half_word,
            TargetRegister::B => self.b = half_word,
            TargetRegister::C => self.c = half_word,
            TargetRegister::D => self.d = half_word,
            TargetRegister::E => self.e = half_word,
            TargetRegister::F => todo!(),
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
            TargetRegister::F => todo!(),
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

/// Flag registers
///Bit  Name  Set Clr  Expl.
/// 7    zf    Z   NZ   Zero Flag
/// 6    n     -   -    Add/Sub-Flag (BCD)
/// 5    h     -   -    Half Carry Flag (BCD)
/// 4    cy    C   NC   Carry Flag
/// 3-0  -     -   -    Not used (always zero)
#[derive(Debug)]
struct FlagRegister {
    z: bool,
    n: bool,
    h: bool,
    c: bool,
}

impl FlagRegister {
    pub fn from_byte(byte: u8) -> FlagRegister {
        FlagRegister {
            z: (byte >> 6) == 1,
            n: (byte >> 5) == 1,
            h: (byte >> 4) == 1,
            c: (byte >> 3) == 1,
        }
    }

    pub fn set_z(&mut self, flag: bool) {
        self.z = flag
    }
    pub fn set_n(&mut self, flag: bool) {
        self.n = flag
    }
    pub fn set_h(&mut self, flag: bool) {
        self.h = flag
    }
    pub fn set_c(&mut self, flag: bool) {
        self.c = flag
    }
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
                f: FlagRegister::from_byte(0x80),
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
                self.ldn_u16(TargetRegister::B, TargetRegister::C, operands)
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

            // 0x7, "RLCA", 0, 1, func(cpu *CPU, operands []byte) { cpu.rlca() }},
            // 0x8, "LD (nn),SP", 2, 5, func(cpu *CPU, operands []byte) { cpu.ldnn_sp(operands) }},
            // 0x9, "ADD HL,BC", 0, 2, func(cpu *CPU, operands []byte) { cpu.addhl_rr(&cpu.Regs.B, &cpu.Regs.C) }},
            // 0xA, "LD A,(BC)", 0, 2, func(cpu *CPU, operands []byte) { cpu.ldr_rr(cpu.Regs.B, cpu.Regs.C, &cpu.Regs.A) }},
            // 0xB, "DEC BC", 0, 2, func(cpu *CPU, operands []byte) { cpu.dec_nn(&cpu.Regs.B, &cpu.Regs.C) }},
            // 0xC, "INC C", 0, 1, func(cpu *CPU, operands []byte) { cpu.inc_n(&cpu.Regs.C) }},
            // 0xD, "DEC C", 0, 1, func(cpu *CPU, operands []byte) { cpu.dec_n(&cpu.Regs.C) }},
            // 0xE, "LD C,n", 1, 2, func(cpu *CPU, operands []byte) { cpu.ldnn_n(&cpu.Regs.C, operands) }},
            0x05 => {
                // DEC B
                self.dec_u8(TargetRegister::B);
            }
            0x06 => {
                // LD B,u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::B, operands)
            }
            0x07 => {
                // RLCA, 0, 1
                self.rlca();
            }
            0x08 => {}
            0x09 => {}
            0x0A => {}
            _ => bail!("not implemented opcode {:X}", opcode),
        }

        Ok(())
    }

    fn ldn_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister, ops: Operands) {
        self.registers.write(reg1, ops[1]);
        self.registers.write(reg2, ops[0]);
    }

    fn ldn_u8(&mut self, reg: TargetRegister, ops: Operands) {
        self.registers.write(reg, ops[0]);
    }

    fn rlca(&self) {}

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
        let i = self.registers.read(reg);
        let incremented = self.inc(i);

        self.registers.write(reg, incremented);
    }

    fn dec_u8(&mut self, reg: TargetRegister) {}

    // TODO 動作が不安なのでテストコード書きたい
    fn inc(&mut self, byte: HalfWord) -> HalfWord {
        let incremented = byte + 1;

        self.registers.f.set_n(false);

        if incremented == 0 {
            self.registers.f.set_z(true);
        } else {
            self.registers.f.set_z(false);
        }

        if byte & 0x10 != 0x10 && incremented & 0x10 == 0x10 {
            self.registers.f.set_h(true);
        } else {
            self.registers.f.set_h(false);
        }

        return incremented;
    }
}

fn join_half_words(upper: HalfWord, lower: HalfWord) -> Word {
    (upper as u16) << 8 ^ lower as u16
}

fn split_word(word: Word) -> (HalfWord, HalfWord) {
    ((word >> 8) as HalfWord, (word ^ 0x00FF) as HalfWord)
}
