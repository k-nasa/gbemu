use crate::{bus::Bus, HalfWord, Word};

type Opecode = u8;

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
    bus: Bus,
}

impl Cpu {
    pub fn new() -> Cpu {
        todo!()
    }

    pub fn step(&mut self) -> Result<(), ()> {
        let opcode = self.fetch();

        let instruction = if opcode == 0xCB {
            let opcode = self.fetch();
            Instruction::resolve_cb_prefix_instruction(opcode)
        } else {
            Instruction::resolve_instruction(opcode)
        };
        let operands = self.fetch_operands(instruction.length_in_bytes);

        instruction.execute(self, &operands);

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
}

type F = fn(&mut Cpu, &[HalfWord]);
const INSTRUCTIONS: &[Instruction] = &[inst(0x01, "NOP", 0, 1, |_, _| {})];
const CB_PREFIX_INSTRUCTIONS: &[Instruction] = &[inst(0x01, "NOP", 0, 1, |_, _| {})];

const fn inst(
    opcode: u8,
    description: &'static str,
    length_in_bytes: usize,
    duration_in_cycle: usize,
    executor: F,
) -> Instruction {
    Instruction {
        opcode,
        description,
        length_in_bytes,
        duration_in_cycle,
        executor,
    }
}

#[derive(Clone)]
struct Instruction {
    opcode: u8,
    description: &'static str,
    length_in_bytes: usize,
    duration_in_cycle: usize,
    executor: F,
}

impl Instruction {
    pub fn resolve_instruction(opcode: HalfWord) -> Instruction {
        INSTRUCTIONS[opcode as usize].clone()
    }

    pub fn resolve_cb_prefix_instruction(opcode: HalfWord) -> Instruction {
        CB_PREFIX_INSTRUCTIONS[opcode as usize].clone()
    }

    pub fn execute(&self, cpu: &mut Cpu, operands: &[HalfWord]) {
        (self.executor)(cpu, operands)
    }
}
