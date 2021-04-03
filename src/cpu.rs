use crate::{bus::Bus, join_half_words, split_word, HalfWord, Word};
use anyhow::Result;

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

    pub fn read(&self, target: TargetRegister) -> HalfWord {
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
            //  ------------ 0x0N ----------------
            0x00 => {} // NOP
            0x01 => {
                // LD BC, u16
                let operands = self.fetch_operands(2);
                self.ldn_u16(TargetRegister::B, TargetRegister::C, operands)
            }
            0x02 => self.ldrr_r(TargetRegister::B, TargetRegister::C, TargetRegister::A), // LD (BC),A
            0x03 => self.inc_u16(TargetRegister::B, TargetRegister::C),                   // INC BC
            0x04 => self.inc_u8(TargetRegister::B),                                       // INC B
            0x05 => self.dec_u8(TargetRegister::B),                                       // DEC B
            0x06 => {
                // LD B,u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::B, operands)
            }
            0x07 => self.rlca(), // RLCA
            0x08 => {
                // LD (u16), SP
                let operands = self.fetch_operands(2);
                self.ldnn_sp(operands);
            }
            0x09 => self.addhl_rr(TargetRegister::B, TargetRegister::C), // ADD HL, BC
            0x0A => self.ldr_rr(TargetRegister::A, TargetRegister::B, TargetRegister::C), // LD A, (BC)
            0x0B => self.dec_u16(TargetRegister::B, TargetRegister::C),                   // DEC BC
            0x0C => self.inc_u8(TargetRegister::C),                                       // INC C
            0x0D => self.dec_u8(TargetRegister::C),                                       // DEC C
            0x0E => {
                // LD C,u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::C, operands)
            }
            0x0F => self.rrca(), // RRCA

            //  ------------ 0X1N ----------------
            0x10 => todo!(), // 0x10, "STOP", 1, 0, func(cpu *CPU, operands []byte) { cpu.stop() }},
            0x11 => todo!(),
            0x12 => todo!(),
            0x13 => todo!(),
            0x14 => todo!(),
            0x15 => todo!(),
            0x16 => todo!(),
            0x17 => todo!(),
            0x18 => todo!(),
            0x19 => todo!(),
            0x1A => todo!(),
            0x1B => todo!(),
            0x1C => todo!(),
            0x1D => todo!(),
            0x1E => todo!(),
            0x1F => todo!(),

            //  ------------ 0X2N ----------------
            0x20 => todo!(),
            0x21 => todo!(),
            0x22 => todo!(),
            0x23 => todo!(),
            0x24 => todo!(),
            0x25 => todo!(),
            0x26 => todo!(),
            0x27 => todo!(),
            0x28 => todo!(),
            0x29 => todo!(),
            0x2A => todo!(),
            0x2B => todo!(),
            0x2C => todo!(),
            0x2D => todo!(),
            0x2E => todo!(),
            0x2F => todo!(),

            //  ------------ 0X3N ----------------
            0x30 => todo!(),
            0x31 => todo!(),
            0x32 => todo!(),
            0x33 => todo!(),
            0x34 => todo!(),
            0x35 => todo!(),
            0x36 => todo!(),
            0x37 => todo!(),
            0x38 => todo!(),
            0x39 => todo!(),
            0x3A => todo!(),
            0x3B => todo!(),
            0x3C => todo!(),
            0x3D => todo!(),
            0x3E => todo!(),
            0x3F => todo!(),

            //  ------------ 0X4N ----------------
            0x40 => todo!(),
            0x41 => todo!(),
            0x42 => todo!(),
            0x43 => todo!(),
            0x44 => todo!(),
            0x45 => todo!(),
            0x46 => todo!(),
            0x47 => todo!(),
            0x48 => todo!(),
            0x49 => todo!(),
            0x4A => todo!(),
            0x4B => todo!(),
            0x4C => todo!(),
            0x4D => todo!(),
            0x4E => todo!(),
            0x4F => todo!(),

            //  ------------ 0X5N ----------------
            0x50 => todo!(),
            0x51 => todo!(),
            0x52 => todo!(),
            0x53 => todo!(),
            0x54 => todo!(),
            0x55 => todo!(),
            0x56 => todo!(),
            0x57 => todo!(),
            0x58 => todo!(),
            0x59 => todo!(),
            0x5A => todo!(),
            0x5B => todo!(),
            0x5C => todo!(),
            0x5D => todo!(),
            0x5E => todo!(),
            0x5F => todo!(),

            //  ------------ 0X6N ----------------
            0x60 => todo!(),
            0x61 => todo!(),
            0x62 => todo!(),
            0x63 => todo!(),
            0x64 => todo!(),
            0x65 => todo!(),
            0x66 => todo!(),
            0x67 => todo!(),
            0x68 => todo!(),
            0x69 => todo!(),
            0x6A => todo!(),
            0x6B => todo!(),
            0x6C => todo!(),
            0x6D => todo!(),
            0x6E => todo!(),
            0x6F => todo!(),

            //  ------------ 0X7N ----------------
            0x70 => todo!(),
            0x71 => todo!(),
            0x72 => todo!(),
            0x73 => todo!(),
            0x74 => todo!(),
            0x75 => todo!(),
            0x76 => todo!(),
            0x77 => todo!(),
            0x78 => todo!(),
            0x79 => todo!(),
            0x7A => todo!(),
            0x7B => todo!(),
            0x7C => todo!(),
            0x7D => todo!(),
            0x7E => todo!(),
            0x7F => todo!(),

            //  ------------ 0X8N ----------------
            0x80 => todo!(),
            0x81 => todo!(),
            0x82 => todo!(),
            0x83 => todo!(),
            0x84 => todo!(),
            0x85 => todo!(),
            0x86 => todo!(),
            0x87 => todo!(),
            0x88 => todo!(),
            0x89 => todo!(),
            0x8A => todo!(),
            0x8B => todo!(),
            0x8C => todo!(),
            0x8D => todo!(),
            0x8E => todo!(),
            0x8F => todo!(),

            //  ------------ 0X9N ----------------
            0x90 => todo!(),
            0x91 => todo!(),
            0x92 => todo!(),
            0x93 => todo!(),
            0x94 => todo!(),
            0x95 => todo!(),
            0x96 => todo!(),
            0x97 => todo!(),
            0x98 => todo!(),
            0x99 => todo!(),
            0x9A => todo!(),
            0x9B => todo!(),
            0x9C => todo!(),
            0x9D => todo!(),
            0x9E => todo!(),
            0x9F => todo!(),

            //  ------------ 0XAN ----------------
            0xA0 => todo!(),
            0xA1 => todo!(),
            0xA2 => todo!(),
            0xA3 => todo!(),
            0xA4 => todo!(),
            0xA5 => todo!(),
            0xA6 => todo!(),
            0xA7 => todo!(),
            0xA8 => todo!(),
            0xA9 => todo!(),
            0xAA => todo!(),
            0xAB => todo!(),
            0xAC => todo!(),
            0xAD => todo!(),
            0xAE => todo!(),
            0xAF => todo!(),

            //  ------------ 0XBN ----------------
            0xB0 => todo!(),
            0xB1 => todo!(),
            0xB2 => todo!(),
            0xB3 => todo!(),
            0xB4 => todo!(),
            0xB5 => todo!(),
            0xB6 => todo!(),
            0xB7 => todo!(),
            0xB8 => todo!(),
            0xB9 => todo!(),
            0xBA => todo!(),
            0xBB => todo!(),
            0xBC => todo!(),
            0xBD => todo!(),
            0xBE => todo!(),
            0xBF => todo!(),

            //  ------------ 0XCN ----------------
            0xC0 => todo!(),
            0xC1 => todo!(),
            0xC2 => todo!(),
            0xC3 => todo!(),
            0xC4 => todo!(),
            0xC5 => todo!(),
            0xC6 => todo!(),
            0xC7 => todo!(),
            0xC8 => todo!(),
            0xC9 => todo!(),
            0xCA => todo!(),
            0xCB => todo!(),
            0xCC => todo!(),
            0xCD => todo!(),
            0xCE => todo!(),
            0xCF => todo!(),

            //  ------------ 0XDN ----------------
            0xD0 => todo!(),
            0xD1 => todo!(),
            0xD2 => todo!(),
            0xD3 => todo!(),
            0xD4 => todo!(),
            0xD5 => todo!(),
            0xD6 => todo!(),
            0xD7 => todo!(),
            0xD8 => todo!(),
            0xD9 => todo!(),
            0xDA => todo!(),
            0xDB => todo!(),
            0xDC => todo!(),
            0xDD => todo!(),
            0xDE => todo!(),
            0xDF => todo!(),

            //  ------------ 0XEN ----------------
            0xE0 => todo!(),
            0xE1 => todo!(),
            0xE2 => todo!(),
            0xE3 => todo!(),
            0xE4 => todo!(),
            0xE5 => todo!(),
            0xE6 => todo!(),
            0xE7 => todo!(),
            0xE8 => todo!(),
            0xE9 => todo!(),
            0xEA => todo!(),
            0xEB => todo!(),
            0xEC => todo!(),
            0xED => todo!(),
            0xEE => todo!(),
            0xEF => todo!(),

            //  ------------ 0XFN ----------------
            0xF0 => todo!(),
            0xF1 => todo!(),
            0xF2 => todo!(),
            0xF3 => todo!(),
            0xF4 => todo!(),
            0xF5 => todo!(),
            0xF6 => todo!(),
            0xF7 => todo!(),
            0xF8 => todo!(),
            0xF9 => todo!(),
            0xFA => todo!(),
            0xFB => todo!(),
            0xFC => todo!(),
            0xFD => todo!(),
            0xFE => todo!(),
            0xFF => todo!(),
            // _ => bail!("not implemented opcode {:X}", opcode),
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

    fn rlca(&mut self) {
        let byte = self.registers.read(TargetRegister::A) << 1;
        let mut shifted = byte << 1;

        // Shift and rotate bits
        if byte & 0x80 == 0x80 {
            self.registers.f.set_c(true);
            shifted = shifted ^ 0x01;
        } else {
            self.registers.f.set_c(false);
        }

        if shifted == 0 {
            self.registers.f.set_z(true);
        }
        self.registers.f.set_n(false);
        self.registers.f.set_h(false);

        self.registers.write(TargetRegister::A, shifted);
    }

    fn rrca(&mut self) {
        let byte = self.registers.read(TargetRegister::A) << 1;
        let mut shifted = byte >> 1;

        // Shift and rotate bits
        if byte & 0x01 == 0x01 {
            self.registers.f.set_c(true);
            shifted = shifted ^ 0x80;
        } else {
            self.registers.f.set_c(false);
        }

        if shifted == 0 {
            self.registers.f.set_z(true);
        }
        self.registers.f.set_n(false);
        self.registers.f.set_h(false);

        self.registers.write(TargetRegister::A, shifted);
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

    fn ldr_rr(
        &mut self,
        dest_reg: TargetRegister,
        upper_reg: TargetRegister,
        lower_reg: TargetRegister,
    ) {
        let address = join_half_words(
            self.registers.read(upper_reg),
            self.registers.read(lower_reg),
        );

        let byte = self.bus.read_byte(address);
        self.registers.write(dest_reg, byte);
    }

    fn inc_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister) {
        let mut word = join_half_words(self.registers.read(reg1), self.registers.read(reg2));
        word += 1;

        let (upper, lower) = split_word(word);

        self.registers.write(reg1, upper);
        self.registers.write(reg2, lower);
    }

    fn dec_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister) {
        let mut word = join_half_words(self.registers.read(reg1), self.registers.read(reg2));
        word -= 1;

        let (upper, lower) = split_word(word);

        self.registers.write(reg1, upper);
        self.registers.write(reg2, lower);
    }

    fn inc_u8(&mut self, reg: TargetRegister) {
        let i = self.registers.read(reg);
        let incremented = self.inc(i);

        self.registers.write(reg, incremented);
    }

    fn dec_u8(&mut self, reg: TargetRegister) {
        let i = self.registers.read(reg);
        let decremented = self.dec(i);

        self.registers.write(reg, decremented);
    }

    fn inc(&mut self, byte: HalfWord) -> HalfWord {
        let incremented = byte + 1;

        self.registers.f.set_n(false);

        if incremented == 0 {
            self.registers.f.set_z(true);
        } else {
            self.registers.f.set_z(false);
        }

        // TODO 動作が不安なのでテストコード書きたい
        if byte & 0x10 != 0x10 && incremented & 0x10 == 0x10 {
            self.registers.f.set_h(true);
        } else {
            self.registers.f.set_h(false);
        }

        return incremented;
    }

    fn dec(&mut self, byte: HalfWord) -> HalfWord {
        let decremented = byte - 1;

        self.registers.f.set_n(true);

        if decremented == 0 {
            self.registers.f.set_z(true);
        } else {
            self.registers.f.set_z(false);
        }

        // TODO 動作が不安なのでテストコード書きたい
        if (decremented ^ 0x01 ^ byte) & 0x10 == 0x10 {
            self.registers.f.set_h(true);
        } else {
            self.registers.f.set_h(false);
        }

        return decremented;
    }

    fn ldnn_sp(&mut self, operands: Operands) {
        let address = join_half_words(operands[1], operands[0]);

        self.bus.write_word(address, self.sp);
    }

    fn addhl_rr(&mut self, upper_reg: TargetRegister, lower_reg: TargetRegister) {
        let hl = join_half_words(
            self.registers.read(TargetRegister::H),
            self.registers.read(TargetRegister::L),
        );

        let rr = join_half_words(
            self.registers.read(upper_reg),
            self.registers.read(lower_reg),
        );

        let result = self.add_words(hl, rr);
        let (upper, lower) = split_word(result);

        self.registers.write(TargetRegister::H, upper);
        self.registers.write(TargetRegister::L, lower);
    }

    fn add_words(&mut self, a: Word, b: Word) -> Word {
        let (added, overflow) = a.overflowing_add(b);

        self.registers.f.set_n(false);

        if overflow {
            self.registers.f.set_c(true)
        } else {
            self.registers.f.set_c(false)
        }

        if added == 0 {
            self.registers.f.set_z(true);
        } else {
            self.registers.f.set_z(false);
        }

        // FIXME わかりやすくしたい。というかあんまり理解できてない
        if (added ^ a ^ b) & 0x1000 == 0x1000 {
            self.registers.f.set_h(true);
        } else {
            self.registers.f.set_h(false);
        }

        return added;
    }
}
