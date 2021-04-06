use crate::logger::Logger;
use crate::SharedBus;
use crate::{join_half_words, split_word, HalfWord, Word};
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

    pub fn get_z(&self) -> bool {
        self.z
    }
    // pub fn get_n(&self) -> bool {
    //     self.n
    // }
    // pub fn get_h(&self) -> bool {
    //     self.h
    // }
    pub fn get_c(&self) -> bool {
        self.c
    }
}

// ref http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
const INIT_PC: Word = 0x100;
const INIT_SP: Word = 0xFFFE;

pub struct Cpu<L>
where
    L: Logger + ?Sized,
{
    logger: Box<L>,
    registers: Registers,
    pc: Word,
    sp: Word,
    bus: SharedBus,

    halted: bool,
}

impl<L> Cpu<L>
where
    L: Logger + ?Sized,
{
    pub fn new(logger: Box<L>, bus: SharedBus) -> Self {
        Cpu {
            logger,
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
            halted: false,
        }
    }

    pub fn step(&mut self) -> Result<()> {
        if self.halted {
            self.logger.info(format!("halted cpu"));
            return Ok(());
        }

        let opcode = self.fetch();

        self.execute(opcode);

        Ok(())
    }

    fn fetch(&mut self) -> Opecode {
        let opcode = self.bus_read_byte(self.pc);
        self.pc += 1;

        opcode
    }

    fn fetch_operands(&mut self, length_in_bytes: usize) -> Vec<u8> {
        (0..length_in_bytes).map(|_| self.fetch()).collect()
    }

    // opcode list https://izik1.github.io/gbops/
    fn execute(&mut self, opcode: Opecode) {
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
            0x11 => {
                // LD DE, u16
                let operands = self.fetch_operands(2);
                self.ldn_u16(TargetRegister::D, TargetRegister::E, operands)
            }
            0x12 => todo!(),
            0x13 => todo!(),
            0x14 => todo!(),
            0x15 => todo!(),
            0x16 => {
                // LD D, u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::D, operands)
            }
            0x17 => todo!(),
            0x18 => {
                // JR i8
                let operands = self.fetch_operands(1);
                self.jr_i8(operands);
            }
            0x19 => todo!(),
            0x1A => self.ldr_rr(TargetRegister::A, TargetRegister::D, TargetRegister::E), // LD A, (DE)
            0x1B => todo!(),
            0x1C => todo!(),
            0x1D => todo!(),
            0x1E => {
                // LD E,u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::E, operands)
            }
            0x1F => todo!(),

            //  ------------ 0X2N ----------------
            0x20 => {
                // JR NZ, u8
                let operands = self.fetch_operands(1);
                self.jrcc_i8(self.registers.f.get_z(), false, operands);
            }
            0x21 => {
                // LD HL, u16
                let operands = self.fetch_operands(2);
                self.ldn_u16(TargetRegister::H, TargetRegister::L, operands)
            }
            0x22 => self.ld_inc_hl_a(),
            // LD (HL+), A
            0x23 => todo!(),
            0x24 => todo!(),
            0x25 => todo!(),
            0x26 => {
                // LD E, u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::E, operands)
            }
            0x27 => todo!(),
            0x28 => {
                // JR Z, u8
                let operands = self.fetch_operands(1);
                self.jrcc_i8(self.registers.f.get_z(), true, operands);
            }
            0x29 => todo!(),
            0x2A => self.ld_inc_a_hl(), // LD A, (HL+)
            0x2B => todo!(),
            0x2C => todo!(),
            0x2D => todo!(),
            0x2E => {
                // LD L,u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::L, operands)
            }
            0x2F => todo!(),

            //  ------------ 0X3N ----------------
            0x30 => {
                // JR NC, u8
                let operands = self.fetch_operands(1);
                self.jrcc_i8(self.registers.f.get_c(), false, operands);
            }
            0x31 => {
                // LD SP, u16
                let operands = self.fetch_operands(2);
                self.ldsp_u16(operands)
            }
            0x32 => self.ld_dec_hl_a(), // LD (HL-),A
            0x33 => todo!(),
            0x34 => todo!(),
            0x35 => todo!(),
            0x36 => {
                // LD (HL),u8 - 0x36
                let operands = self.fetch_operands(1);
                self.ldrr_u8(TargetRegister::H, TargetRegister::L, operands);
            }
            0x37 => todo!(),
            0x38 => {
                // JR C, u8
                let operands = self.fetch_operands(1);
                self.jrcc_i8(self.registers.f.get_c(), true, operands);
            }
            0x39 => todo!(),
            0x3A => self.ld_dec_a_hl(), // LD A, (HL-)
            0x3B => todo!(),
            0x3C => todo!(),
            0x3D => todo!(),
            0x3E => {
                // LD A,u8
                let operands = self.fetch_operands(1);
                self.ldn_u8(TargetRegister::A, operands)
            }
            0x3F => todo!(),

            //  ------------ 0X4N ----------------
            0x40 => self.ldrr(TargetRegister::B, TargetRegister::B), // LD B, B
            0x41 => self.ldrr(TargetRegister::B, TargetRegister::C), // LD B, C
            0x42 => self.ldrr(TargetRegister::B, TargetRegister::D), // LD B, D
            0x43 => self.ldrr(TargetRegister::B, TargetRegister::E), // LD B, E
            0x44 => self.ldrr(TargetRegister::B, TargetRegister::H), // LD B, H
            0x45 => self.ldrr(TargetRegister::B, TargetRegister::L), // LD B, L
            0x46 => self.ldr_rr(TargetRegister::B, TargetRegister::H, TargetRegister::L), // LD B,(HL)

            0x47 => self.ldrr(TargetRegister::B, TargetRegister::A), // LD B, A
            0x48 => self.ldrr(TargetRegister::C, TargetRegister::B), // LD C, B
            0x49 => self.ldrr(TargetRegister::C, TargetRegister::C), // LD C, C
            0x4A => self.ldrr(TargetRegister::C, TargetRegister::D), // LD C, D
            0x4B => self.ldrr(TargetRegister::C, TargetRegister::E), // LD C, E
            0x4C => self.ldrr(TargetRegister::C, TargetRegister::H), // LD C, H
            0x4D => self.ldrr(TargetRegister::C, TargetRegister::L), // LD C, L
            0x4E => self.ldr_rr(TargetRegister::C, TargetRegister::H, TargetRegister::L), // LD C,(HL)
            0x4F => self.ldrr(TargetRegister::C, TargetRegister::A),                      // LD C, A

            //  ------------ 0X5N ----------------
            0x50 => self.ldrr(TargetRegister::D, TargetRegister::B), // LD D, B
            0x51 => self.ldrr(TargetRegister::D, TargetRegister::C), // LD D, C
            0x52 => self.ldrr(TargetRegister::D, TargetRegister::H), // LD D, D
            0x53 => self.ldrr(TargetRegister::D, TargetRegister::E), // LD D, E
            0x54 => self.ldrr(TargetRegister::D, TargetRegister::H), // LD D, H
            0x55 => self.ldrr(TargetRegister::D, TargetRegister::L), // LD D, L
            0x56 => self.ldr_rr(TargetRegister::D, TargetRegister::H, TargetRegister::L), // LD D,(HL)

            0x57 => self.ldrr(TargetRegister::D, TargetRegister::A), // LD D, A
            0x58 => self.ldrr(TargetRegister::E, TargetRegister::B), // LD E, B
            0x59 => self.ldrr(TargetRegister::E, TargetRegister::C), // LD E, C
            0x5A => self.ldrr(TargetRegister::E, TargetRegister::H), // LD E, D
            0x5B => self.ldrr(TargetRegister::E, TargetRegister::E), // LD E, E
            0x5C => self.ldrr(TargetRegister::E, TargetRegister::H), // LD E, H
            0x5D => self.ldrr(TargetRegister::E, TargetRegister::L), // LD E, L
            0x5E => self.ldr_rr(TargetRegister::E, TargetRegister::H, TargetRegister::L), // LD E,(HL)
            0x5F => self.ldrr(TargetRegister::E, TargetRegister::A),                      // LD E, A

            //  ------------ 0X6N ----------------
            0x60 => self.ldrr(TargetRegister::H, TargetRegister::B), // LD H, B
            0x61 => self.ldrr(TargetRegister::H, TargetRegister::C), // LD H, C
            0x62 => self.ldrr(TargetRegister::H, TargetRegister::D), // LD H, D
            0x63 => self.ldrr(TargetRegister::H, TargetRegister::E), // LD H, E
            0x64 => self.ldrr(TargetRegister::H, TargetRegister::H), // LD H, H
            0x65 => self.ldrr(TargetRegister::H, TargetRegister::L), // LD H, L
            0x66 => self.ldr_rr(TargetRegister::H, TargetRegister::H, TargetRegister::L), // LD H,(HL)
            0x67 => self.ldrr(TargetRegister::H, TargetRegister::A),                      // LD H, A
            0x68 => self.ldrr(TargetRegister::L, TargetRegister::B),                      // LD L, B
            0x69 => self.ldrr(TargetRegister::L, TargetRegister::C),                      // LD L, C
            0x6A => self.ldrr(TargetRegister::L, TargetRegister::D),                      // LD L, D
            0x6B => self.ldrr(TargetRegister::L, TargetRegister::E),                      // LD L, E
            0x6C => self.ldrr(TargetRegister::L, TargetRegister::H),                      // LD L, H
            0x6D => self.ldrr(TargetRegister::L, TargetRegister::L),                      // LD L, L
            0x6E => self.ldr_rr(TargetRegister::L, TargetRegister::H, TargetRegister::L), // LD L,(HL)
            0x6F => self.ldrr(TargetRegister::L, TargetRegister::A),                      // LD L, A

            //  ------------ 0X7N ----------------
            0x70 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::B), // LD (HL),B
            0x71 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::C), // LD (HL),C
            0x72 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::D), // LD (HL),D
            0x73 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::E), // LD (HL),E
            0x74 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::H), // LD (HL),H
            0x75 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::L), // LD (HL),L
            0x76 => self.halt(),                                                          // HALT
            0x77 => self.ldrr_r(TargetRegister::H, TargetRegister::L, TargetRegister::A), // LD (HL),A

            0x78 => self.ldrr(TargetRegister::A, TargetRegister::B), // LD A, B
            0x79 => self.ldrr(TargetRegister::A, TargetRegister::C), // LD A, C
            0x7A => self.ldrr(TargetRegister::A, TargetRegister::D), // LD A, D
            0x7B => self.ldrr(TargetRegister::A, TargetRegister::E), // LD A, E
            0x7C => self.ldrr(TargetRegister::A, TargetRegister::H), // LD A, H
            0x7D => self.ldrr(TargetRegister::A, TargetRegister::L), // LD A, L
            0x7E => self.ldr_rr(TargetRegister::A, TargetRegister::H, TargetRegister::L), // LD A, (HL)
            0x7F => self.ldrr(TargetRegister::A, TargetRegister::A),                      // LD A, A

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
            0xA8 => self.xora_r(TargetRegister::B), // XOR A, B
            0xA9 => self.xora_r(TargetRegister::C), // XOR A, C
            0xAA => self.xora_r(TargetRegister::D), // XOR A, D
            0xAB => self.xora_r(TargetRegister::E), // XOR A, E
            0xAC => self.xora_r(TargetRegister::H), // XOR A, H
            0xAD => self.xora_r(TargetRegister::L), // XOR A, L
            0xAE => self.xora_u16(self.read_hl()),  // XOR A, (HL)
            0xAF => self.xora_r(TargetRegister::A), // XOR A, A

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
            0xC0 => self.retcc(self.registers.f.get_z(), false), // RET NZ
            0xC1 => todo!(),
            0xC2 => todo!(),
            0xC3 => {
                // JP u16
                let operands = self.fetch_operands(2);
                self.jp_u16(operands);
            }
            0xC4 => {
                // CALL NZ, u16 - 0xCD
                let operands = self.fetch_operands(2);
                self.callcc_u16(self.registers.f.get_z(), false, operands);
            }
            0xC5 => todo!(),
            0xC6 => todo!(),
            0xC7 => todo!(),
            0xC8 => self.retcc(self.registers.f.get_z(), true), // RET Z
            0xC9 => self.ret(),                                 // RET
            0xCA => todo!(),
            0xCB => todo!(),
            0xCC => {
                // CALL Z, u16
                let operands = self.fetch_operands(2);
                self.callcc_u16(self.registers.f.get_z(), true, operands);
            }
            0xCD => {
                // CALL u16 - 0xCD
                let operands = self.fetch_operands(2);
                self.call_u16(operands);
            }
            0xCE => todo!(),
            0xCF => todo!(),

            //  ------------ 0XDN ----------------
            0xD0 => self.retcc(self.registers.f.get_c(), false), // RET NC
            0xD1 => todo!(),
            0xD2 => todo!(),
            0xD3 => todo!(),
            0xD4 => {
                // CALL NC, u16 - 0xCD
                let operands = self.fetch_operands(2);
                self.callcc_u16(self.registers.f.get_c(), false, operands);
            }
            0xD5 => todo!(),
            0xD6 => todo!(),
            0xD7 => todo!(),
            0xD8 => self.retcc(self.registers.f.get_c(), true), // RET C
            0xD9 => todo!(),
            0xDA => todo!(),
            0xDB => todo!(),
            0xDC => {
                // CALL C, u16 - 0xCD
                let operands = self.fetch_operands(2);
                self.callcc_u16(self.registers.f.get_c(), true, operands);
            }
            0xDD => todo!(),
            0xDE => todo!(),
            0xDF => todo!(),

            //  ------------ 0XEN ----------------
            0xE0 => {
                // LD (FF00+u8),A
                let operands = self.fetch_operands(1);
                self.ldn_a(operands);
            }
            0xE1 => todo!(),
            0xE2 => self.ldc_a(), // LD (0xFF00+C),A
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
            0xF0 => {
                // LD A (0xFF00 + u8)
                let operands = self.fetch_operands(1);
                self.ldu8_a(operands);
            }
            0xF1 => todo!(),
            0xF2 => self.lda_c(), // LD A, (0xFF00+C)
            0xF3 => { /*TODO 割り込み処理を実装したらDIも実装する*/ } // DI disable intruppt
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
            0xFE => {
                // CP A, u8
                let operands = self.fetch_operands(1);
                self.cp_u8(operands);
            }
            0xFF => todo!(),
            // _ => bail!("not implemented opcode {:X}", opcode),
        }
    }

    fn ldn_u16(&mut self, reg1: TargetRegister, reg2: TargetRegister, ops: Operands) {
        self.registers.write(reg1, ops[1]);
        self.registers.write(reg2, ops[0]);
    }

    fn ldn_u8(&mut self, reg: TargetRegister, ops: Operands) {
        self.registers.write(reg, ops[0]);
    }

    fn ldn_a(&mut self, operands: Operands) {
        self.bus_write_byte(
            0xFF00 + operands[0] as u16,
            self.registers.read(TargetRegister::A),
        )
    }

    fn ldu8_a(&mut self, operands: Operands) {
        let byte = self.bus_read_byte(0xFF00 + operands[0] as u16);
        self.registers.write(TargetRegister::A, byte);
    }

    fn ldc_a(&mut self) {
        self.bus_write_byte(
            0xFF00 + self.registers.read(TargetRegister::C) as u16,
            self.registers.read(TargetRegister::A),
        )
    }

    fn lda_c(&mut self) {
        let byte = self.bus_read_byte(0xFF00 + self.registers.read(TargetRegister::C) as u16);
        self.registers.write(TargetRegister::A, byte);
    }

    fn rlca(&mut self) {
        let byte = self.registers.read(TargetRegister::A) << 1;
        let mut shifted = byte << 1;

        // Shift and rotate bits
        if byte & 0x80 == 0x80 {
            self.registers.f.set_c(true);
            shifted ^= 0x01;
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
            shifted ^= 0x80;
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

    fn ldrr(&mut self, dest_reg: TargetRegister, source_reg: TargetRegister) {
        let byte = self.registers.read(source_reg);
        self.registers.write(dest_reg, byte);
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
        self.bus_write_byte(address, byte);
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

        let byte = self.bus_read_byte(address);
        self.registers.write(dest_reg, byte);
    }

    fn ldrr_u8(
        &mut self,
        upper_reg: TargetRegister,
        lower_reg: TargetRegister,
        operands: Operands,
    ) {
        let address = join_half_words(
            self.registers.read(upper_reg),
            self.registers.read(lower_reg),
        );

        self.bus_write_byte(address, operands[0]);
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

        incremented
    }

    fn dec(&mut self, byte: HalfWord) -> HalfWord {
        let decremented = byte.wrapping_sub(1);

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

        decremented
    }

    fn ldnn_sp(&mut self, operands: Operands) {
        let address = join_half_words(operands[1], operands[0]);

        self.bus_write_word(address, self.sp);
    }

    fn addhl_rr(&mut self, upper_reg: TargetRegister, lower_reg: TargetRegister) {
        let hl = self.read_hl();

        let rr = join_half_words(
            self.registers.read(upper_reg),
            self.registers.read(lower_reg),
        );

        let result = self.add_words(hl, rr);
        self.set_hl(result);
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

        added
    }

    fn ldsp_u16(&mut self, operands: Operands) {
        self.sp = join_half_words(operands[1], operands[0])
    }

    fn jp_u16(&mut self, operands: Operands) {
        self.pc = join_half_words(operands[1], operands[0])
    }

    // fn lda_u8(&mut self, operands: Operands) {
    //     let byte = self.bus.bus_read_byte(0xFF00 + operands[0] as u16);
    //     self.registers.write(TargetRegister::A, byte);
    // }

    fn cp_u8(&mut self, operands: Operands) {
        self.registers.f.set_n(true);

        let value = operands[0];
        let a = self.registers.read(TargetRegister::A);

        if a & 0xF < value & 0xF {
            self.registers.f.set_h(true)
        } else {
            self.registers.f.set_h(false)
        }

        if a < value {
            self.registers.f.set_c(true)
        } else {
            self.registers.f.set_c(false)
        }

        if value == a {
            self.registers.f.set_z(true)
        } else {
            self.registers.f.set_z(false)
        }
    }

    fn jrcc_i8(&mut self, flag: bool, is_set: bool, operands: Operands) {
        let n = operands[0] as i8;

        if flag == is_set {
            if n < 0 {
                self.pc -= -n as u16;
            } else {
                self.pc += n as u16;
            }
        }
    }

    fn jr_i8(&mut self, operands: Operands) {
        let n = operands[0] as i8;

        if n < 0 {
            self.pc -= -n as u16;
        } else {
            self.pc += n as u16;
        }
    }

    fn ld_inc_hl_a(&mut self) {
        let mut addr = self.read_hl();

        self.bus_write_byte(addr, self.registers.read(TargetRegister::A));
        addr += 1;

        self.set_hl(addr);
    }

    fn ld_dec_hl_a(&mut self) {
        let mut addr = self.read_hl();

        self.bus_write_byte(addr, self.registers.read(TargetRegister::A));
        addr -= 1;

        self.set_hl(addr);
    }

    fn ld_inc_a_hl(&mut self) {
        let mut addr = self.read_hl();

        let byte = self.bus_read_byte(addr);
        self.registers.write(TargetRegister::A, byte);
        addr += 1;

        self.set_hl(addr);
    }

    fn ld_dec_a_hl(&mut self) {
        let mut addr = self.read_hl();

        let byte = self.bus_read_byte(addr);
        self.registers.write(TargetRegister::A, byte);
        addr -= 1;

        self.set_hl(addr);
    }

    fn xora_r(&mut self, reg: TargetRegister) {
        let byte = self.xor(
            self.registers.read(reg),
            self.registers.read(TargetRegister::A),
        );

        self.registers.write(TargetRegister::A, byte);
    }

    fn xora_u16(&mut self, addr: Word) {
        let value = self.bus_read_byte(addr);
        let byte = self.xor(self.registers.read(TargetRegister::A), value);

        self.registers.write(TargetRegister::A, byte);
    }

    fn xor(&mut self, a: HalfWord, b: HalfWord) -> HalfWord {
        self.registers.f.set_h(false);
        self.registers.f.set_n(false);
        self.registers.f.set_c(false);

        let bit = a ^ b;
        if bit == 0 {
            self.registers.f.set_z(true)
        } else {
            self.registers.f.set_z(false)
        }

        bit
    }

    fn ret(&mut self) {
        let (upper, lower) = (self.pop(), self.pop());

        self.pc = join_half_words(upper, lower);
    }

    fn retcc(&mut self, flag: bool, is_set: bool) {
        if flag == is_set {
            self.ret();
        }
    }

    fn call_u16(&mut self, operands: Operands) {
        let (upper, lower) = (self.pc >> 8, self.pc & 0xFF);
        self.push(upper as u8);
        self.push(lower as u8);

        self.pc = join_half_words(operands[1], operands[0])
    }

    fn callcc_u16(&mut self, flag: bool, is_set: bool, operands: Operands) {
        if flag == is_set {
            self.call_u16(operands);
        }
    }

    fn push(&mut self, half_word: HalfWord) {
        self.sp -= 1;
        self.bus_write_byte(self.sp, half_word)
    }

    fn pop(&mut self) -> HalfWord {
        let byte = self.bus_read_byte(self.sp);
        self.sp += 1;

        byte
    }

    fn read_hl(&self) -> Word {
        join_half_words(
            self.registers.read(TargetRegister::H),
            self.registers.read(TargetRegister::L),
        )
    }

    fn set_hl(&mut self, word: Word) {
        let (upper, lower) = split_word(word);

        self.registers.write(TargetRegister::H, upper);
        self.registers.write(TargetRegister::L, lower);
    }

    fn halt(&mut self) {
        self.halted = true
    }

    pub fn bus_read_byte(&self, address: Word) -> u8 {
        let bus = self.bus.lock().unwrap();
        bus.read_byte(address)
    }

    pub fn bus_write_byte(&mut self, address: Word, byte: HalfWord) {
        let mut bus = self.bus.lock().unwrap();
        bus.write_byte(address, byte)
    }

    pub fn bus_write_word(&mut self, address: Word, word: Word) {
        let mut bus = self.bus.lock().unwrap();
        bus.write_word(address, word)
    }
}
