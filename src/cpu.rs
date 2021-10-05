mod bus;
mod instruction;
mod register;

use bus::Bus;
use instruction::{ArithmeticTarget, Instruction};
use register::Registers;

macro_rules! run_instruction_in_register {
    ($register: ident, $self:ident, $instruction:ident) => {{
        let value = $self.registers.$register;
        let new_value = $self.$instruction(value);
        $self.registers.a = new_value;
        // compute next PC value
        // modulo operation to avoid overflowing effects
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! arithmetic_instruction {
    ($target: ident, $self:ident.$instruction:ident) => {{
        match $target {
            ArithmeticTarget::A => run_instruction_in_register!(a, $self, $instruction),
            ArithmeticTarget::B => run_instruction_in_register!(b, $self, $instruction),
            ArithmeticTarget::C => run_instruction_in_register!(c, $self, $instruction),
            ArithmeticTarget::D => run_instruction_in_register!(d, $self, $instruction),
            ArithmeticTarget::E => run_instruction_in_register!(e, $self, $instruction),
            ArithmeticTarget::H => run_instruction_in_register!(h, $self, $instruction),
            ArithmeticTarget::L => run_instruction_in_register!(l, $self, $instruction),
            ArithmeticTarget::HL => {
                let address = $self.registers.read_hl();
                let value = $self.bus.read_byte(address);
                let new_value = $self.$instruction(value);
                $self.registers.a = new_value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            ArithmeticTarget::D8 => {
                let address = $self.pc.wrapping_add(1);
                let value = $self.bus.read_byte(address);
                let new_value = $self.$instruction(value);
                $self.registers.a = new_value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(2)
            }
        }
    }};
}

pub struct Cpu {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: Bus,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: 0x0000,
            sp: 0x0000,
            bus: Bus::new(),
        }
    }

    fn run(&mut self) {
        // fetch instruction
        let instruction_byte = self.bus.read_byte(self.pc);
        // decode instruction
        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte) {
            // execute instruction
            self.execute(instruction)
        } else {
            panic!("Unknown instruction found for 0x{:x}", instruction_byte);
        };

        // update PC value
        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => arithmetic_instruction!(target, self.add),
            Instruction::ADDC(target) => arithmetic_instruction!(target, self.addc),
            Instruction::SUB(target) => arithmetic_instruction!(target, self.sub),
            Instruction::SBC(target) => arithmetic_instruction!(target, self.subc),
            Instruction::AND(target) => arithmetic_instruction!(target, self.and),
            Instruction::XOR(target) => arithmetic_instruction!(target, self.xor),
            Instruction::OR(target) => arithmetic_instruction!(target, self.or),
            Instruction::CP(target) => arithmetic_instruction!(target, self.cp),
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = false;
        self.registers.f.carry = overflow;
        // Half Carry is set if adding the lower bits of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    fn addc(&mut self, value: u8) -> u8 {
        let carry = self.registers.f.carry as u8;
        let (intermediate_value, first_overflow) = value.overflowing_add(carry as u8);
        let (new_value, second_overflow) = self.registers.a.overflowing_add(intermediate_value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = false;
        self.registers.f.carry = first_overflow || second_overflow;
        // Half Carry is set if adding the lower bits of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) + carry > 0xF;
        new_value
    }

    fn sub(&mut self, value: u8) -> u8 {
        let (new_value, overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = true;
        self.registers.f.carry = overflow;
        // Half Carry is set if adding the lower bits of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) < (value & 0xF);
        new_value
    }

    fn subc(&mut self, value: u8) -> u8 {
        let carry = self.registers.f.carry as u8;
        let (intermediate_value, first_overflow) = value.overflowing_sub(carry);
        let (new_value, second_overflow) = self.registers.a.overflowing_sub(intermediate_value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = true;
        self.registers.f.carry = first_overflow || second_overflow;
        // Half Carry is set if adding the lower bits of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) < (value & 0xF) + carry;
        new_value
    }

    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = false;
        self.registers.f.half_carry = true;
        self.registers.f.carry = false;
        new_value
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        new_value
    }

    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        new_value
    }

    fn cp(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a;
        self.registers.f.zero = self.registers.a == value;
        self.registers.f.substraction = true;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (value & 0xF);
        self.registers.f.carry = self.registers.a < value;
        new_value
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::*;
    use crate::cpu::instruction::ArithmeticTarget::{B, C, D8, HL};
    use crate::cpu::instruction::Instruction::{ADD, ADDC, AND, CP, OR, SBC, SUB, XOR};

    #[test]
    fn test_add_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0xAABB);
        cpu.execute(ADD(B));
        assert_eq!(cpu.registers.read_af(), 0xAA00);
    }

    #[test]
    fn test_add_memory() {
        let mut cpu = Cpu::new();
        let address = 0x1234;
        let data = 0xAA;

        cpu.bus.write_byte(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(ADD(HL));
        assert_eq!(cpu.registers.read_af(), 0xAA00);
    }

    #[test]
    fn test_add_immediate() {
        let mut cpu = Cpu::new();
        let address = 0x0001;
        let data = 0x23;

        cpu.bus.write_byte(address, data);
        cpu.execute(ADD(D8));
        assert_eq!(cpu.registers.read_af(), 0x2300);
    }

    #[test]
    fn test_addc_registers() {
        let mut cpu = Cpu::new();

        cpu.registers.write_af(0x0110);
        cpu.registers.write_bc(0xAABB);
        cpu.execute(ADDC(B));
        assert_eq!(cpu.registers.read_af(), 0xAC00);

        cpu.registers.write_af(0x0110);
        cpu.registers.write_bc(0xFF00);
        cpu.execute(ADDC(B));
        assert_eq!(cpu.registers.read_af(), 0x0130);
    }

    #[test]
    fn test_addc_memory() {
        let mut cpu = Cpu::new();
        let address = 0x1234;
        let data = 0xAA;

        cpu.bus.write_byte(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(ADDC(HL));
        assert_eq!(cpu.registers.read_af(), 0xAA00);
    }

    #[test]
    fn test_addc_immediate() {
        let mut cpu = Cpu::new();
        let address = 0x0001;
        let data = 0x23;

        cpu.bus.write_byte(address, data);
        cpu.registers.write_af(0x0110);
        cpu.execute(ADDC(D8));
        assert_eq!(cpu.registers.read_af(), 0x2500);
    }

    #[test]
    fn test_sub_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0xAABB);
        cpu.registers.write_af(0xFF00);
        cpu.execute(SUB(C));
        assert_eq!(cpu.registers.read_af(), 0x4440);
    }

    #[test]
    fn test_subc_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0xAABB);
        cpu.registers.write_af(0xFF10);
        cpu.execute(SBC(C));
        assert_eq!(cpu.registers.read_af(), 0x4540);
    }

    #[test]
    fn test_and_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0xAABB);
        cpu.registers.write_af(0xAA00);
        cpu.execute(AND(B));
        assert_eq!(cpu.registers.read_af(), 0xAA20);
    }

    #[test]
    fn test_xor_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x0022);
        cpu.registers.write_af(0x2100);
        cpu.execute(XOR(C));
        assert_eq!(cpu.registers.read_af(), 0x0300);
    }

    #[test]
    fn test_or_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x0022);
        cpu.registers.write_af(0x2100);
        cpu.execute(OR(C));
        assert_eq!(cpu.registers.read_af(), 0x2300);
    }

    #[test]
    fn test_cp_registers() {
        let mut cpu = Cpu::new();

        cpu.registers.write_bc(0x0022);
        cpu.registers.write_af(0x2200);
        cpu.execute(CP(C));
        assert_eq!(cpu.registers.read_af(), 0x22C0);

        cpu.registers.write_bc(0x0033);
        cpu.registers.write_af(0x2200);
        cpu.execute(CP(C));
        assert_eq!(cpu.registers.read_af(), 0x2270);
    }
}
