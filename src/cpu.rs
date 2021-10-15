mod bus;
mod instruction;
mod register;

use bus::Bus;
use instruction::{ArithmeticTarget, IncDecTarget, Instruction, U16Target};
use register::Registers;

macro_rules! run_instruction_in_register {
    ($register_in: ident => $register_out: ident, $self:ident.$instruction:ident) => {{
        let value = $self.registers.$register_in;
        let new_value = $self.$instruction(value);
        $self.registers.$register_out = new_value;
        // compute next PC value
        // modulo operation to avoid overflowing effects
        $self.pc.wrapping_add(1)
    }};

    ($read_reg: ident, $write_reg: ident, $self:ident.$instruction:ident) => {{
        let value_in_register = $self.registers.$read_reg();
        let new_value = $self.$instruction(value_in_register);
        $self.registers.$write_reg(new_value);
        // compute next PC value
        // modulo operation to avoid overflowing effects
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! arithmetic_instruction {
    ($target: ident, $self:ident.$instruction:ident) => {{
        match $target {
            ArithmeticTarget::A => run_instruction_in_register!(a => a, $self.$instruction),
            ArithmeticTarget::B => run_instruction_in_register!(b => a, $self.$instruction),
            ArithmeticTarget::C => run_instruction_in_register!(c => a, $self.$instruction),
            ArithmeticTarget::D => run_instruction_in_register!(d => a, $self.$instruction),
            ArithmeticTarget::E => run_instruction_in_register!(e => a, $self.$instruction),
            ArithmeticTarget::H => run_instruction_in_register!(h => a, $self.$instruction),
            ArithmeticTarget::L => run_instruction_in_register!(l => a, $self.$instruction),
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

    ($target: ident => $flag:ident => $self:ident.$instruction:ident) => {{
        match $target {
            U16Target::BC => run_instruction_in_register!(read_bc, write_hl, $self.$instruction),
            U16Target::DE => run_instruction_in_register!(read_de, write_hl, $self.$instruction),
            U16Target::HL => run_instruction_in_register!(read_hl, write_hl, $self.$instruction),
            U16Target::SP => {
                let value_in_register = $self.sp;
                let new_value = $self.$instruction(value_in_register);
                $self.registers.write_hl(new_value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
        }
    }};
}

macro_rules! inc_dec_instruction {
    ($target: ident, $self:ident.$instruction:ident) => {{
        match $target {
            IncDecTarget::A => run_instruction_in_register!(a => a, $self.$instruction),
            IncDecTarget::B => run_instruction_in_register!(b => b, $self.$instruction),
            IncDecTarget::C => run_instruction_in_register!(c => c, $self.$instruction),
            IncDecTarget::D => run_instruction_in_register!(d => d, $self.$instruction),
            IncDecTarget::E => run_instruction_in_register!(e => e, $self.$instruction),
            IncDecTarget::H => run_instruction_in_register!(h => h, $self.$instruction),
            IncDecTarget::L => run_instruction_in_register!(l => l, $self.$instruction),
            IncDecTarget::HL => {
                let address = $self.registers.read_hl();
                let value = $self.bus.read_byte(address);
                let new_value = $self.$instruction(value);
                $self.bus.write_byte(address, new_value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
        }
    }};

    ($target: ident => $flag:ident => $self:ident.$instruction:ident) => {{
        match $target {
            U16Target::BC => run_instruction_in_register!(read_bc, write_bc, $self.$instruction),
            U16Target::DE => run_instruction_in_register!(read_de, write_de, $self.$instruction),
            U16Target::HL => run_instruction_in_register!(read_hl, write_hl, $self.$instruction),
            U16Target::SP => {
                let value_in_register = $self.sp;
                let new_value = $self.$instruction(value_in_register);
                $self.sp = new_value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
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
            Instruction::ADD16(target) => arithmetic_instruction!(target => u16 => self.add16),
            Instruction::ADDC(target) => arithmetic_instruction!(target, self.addc),
            Instruction::SUB(target) => arithmetic_instruction!(target, self.sub),
            Instruction::SBC(target) => arithmetic_instruction!(target, self.subc),
            Instruction::AND(target) => arithmetic_instruction!(target, self.and),
            Instruction::XOR(target) => arithmetic_instruction!(target, self.xor),
            Instruction::OR(target) => arithmetic_instruction!(target, self.or),
            Instruction::CP(target) => arithmetic_instruction!(target, self.cp),
            Instruction::INC(target) => inc_dec_instruction!(target, self.inc),
            Instruction::INC16(target) => inc_dec_instruction!(target => u16 => self.inc16),
            Instruction::DEC(target) => inc_dec_instruction!(target, self.dec),
            Instruction::DEC16(target) => inc_dec_instruction!(target => u16 => self.dec16),
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

    fn add16(&mut self, value: u16) -> u16 {
        let hl_value = self.registers.read_hl();
        let (new_value, overflow) = hl_value.overflowing_add(value);
        self.registers.f.substraction = false;
        self.registers.f.carry = overflow;
        // Half carry tests if we flow over the 11th bit i.e. does adding the two
        // numbers together cause the 11th bit to flip
        let mask = 0b111_1111_1111; // mask out bits 11-15
        self.registers.f.half_carry = (value & mask) + (hl_value & mask) > mask;
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

    fn inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = false;
        self.registers.f.half_carry = (value & 0xF) == 0xF;
        new_value
    }

    fn inc16(&mut self, value: u16) -> u16 {
        let new_value = value.wrapping_add(1);
        new_value
    }

    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.substraction = true;
        self.registers.f.half_carry = (value & 0xF) == 0;
        new_value
    }

    fn dec16(&mut self, value: u16) -> u16 {
        let new_value = value.wrapping_sub(1);
        new_value
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::*;
    use crate::cpu::instruction::ArithmeticTarget::{B, C, D8, HL};
    use crate::cpu::instruction::Instruction::{
        ADD, ADD16, ADDC, AND, CP, DEC, DEC16, INC, INC16, OR, SBC, SUB, XOR,
    };
    use crate::cpu::instruction::{IncDecTarget, U16Target};

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
    fn test_add16_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x2200);
        cpu.registers.write_hl(0x0125);
        cpu.execute(ADD16(U16Target::BC));
        assert_eq!(cpu.registers.read_hl(), 0x2325);

        cpu.registers.write_de(0x00FF);
        cpu.registers.write_hl(0xFF01);
        cpu.execute(ADD16(U16Target::DE));
        assert_eq!(cpu.registers.read_hl(), 0x0000);

        cpu.registers.write_hl(0xF025);
        cpu.execute(ADD16(U16Target::HL));
        assert_eq!(cpu.registers.read_hl(), 0xE04A);

        cpu.sp = 0x0001;
        cpu.registers.write_hl(0xF025);
        cpu.execute(ADD16(U16Target::SP));
        assert_eq!(cpu.registers.read_hl(), 0xF026);
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

    #[test]
    fn test_inc_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x2200);
        cpu.execute(INC(IncDecTarget::B));
        assert_eq!(cpu.registers.read_bc(), 0x2300);

        cpu.registers.write_bc(0x22FF);
        cpu.execute(INC(IncDecTarget::C));
        assert_eq!(cpu.registers.read_bc(), 0x2200);

        let address = 0x1234;
        let data = 0xAA;
        cpu.bus.write_byte(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(INC(IncDecTarget::HL));
        assert_eq!(cpu.bus.read_byte(address), 0xAB);
    }

    #[test]
    fn test_inc16_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x2200);
        cpu.execute(INC16(U16Target::BC));
        assert_eq!(cpu.registers.read_bc(), 0x2201);

        cpu.registers.write_de(0x22FF);
        cpu.execute(INC16(U16Target::DE));
        assert_eq!(cpu.registers.read_de(), 0x2300);

        cpu.registers.write_hl(0xFFFF);
        cpu.execute(INC16(U16Target::HL));
        assert_eq!(cpu.registers.read_hl(), 0x0000);

        cpu.sp = 0x3578;
        cpu.execute(INC16(U16Target::SP));
        assert_eq!(cpu.sp, 0x3579);
    }

    #[test]
    fn test_dec_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x2200);
        cpu.execute(DEC(IncDecTarget::B));
        assert_eq!(cpu.registers.read_bc(), 0x2100);

        cpu.registers.write_bc(0x2200);
        cpu.execute(DEC(IncDecTarget::C));
        assert_eq!(cpu.registers.read_bc(), 0x22FF);

        let address = 0x1234;
        let data = 0xAA;
        cpu.bus.write_byte(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(DEC(IncDecTarget::HL));
        assert_eq!(cpu.bus.read_byte(address), 0xA9);
    }

    #[test]
    fn test_dec16_registers() {
        let mut cpu = Cpu::new();
        cpu.registers.write_bc(0x2200);
        cpu.execute(DEC16(U16Target::BC));
        assert_eq!(cpu.registers.read_bc(), 0x21FF);

        cpu.registers.write_de(0x0000);
        cpu.execute(DEC16(U16Target::DE));
        assert_eq!(cpu.registers.read_de(), 0xFFFF);

        cpu.registers.write_hl(0x1279);
        cpu.execute(DEC16(U16Target::HL));
        assert_eq!(cpu.registers.read_hl(), 0x1278);

        cpu.sp = 0x0001;
        cpu.execute(DEC16(U16Target::SP));
        assert_eq!(cpu.sp, 0x0000);
    }
}
