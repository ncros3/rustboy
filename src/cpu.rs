mod bus;
mod instruction;
mod register;

use bus::Bus;
use instruction::{
    ArithmeticTarget, IncDecTarget, Instruction, JumpImmediateTarget, JumpRelativeTarget,
    Load16Target, U16Target,
};
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

    ($read_reg: ident => $target_type: ident => $write_reg: ident, $self:ident.$instruction:ident) => {{
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
            U16Target::BC => run_instruction_in_register!(read_bc => u16 => write_hl, $self.$instruction),
            U16Target::DE => run_instruction_in_register!(read_de => u16 => write_hl, $self.$instruction),
            U16Target::HL => run_instruction_in_register!(read_hl => u16 => write_hl, $self.$instruction),
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
            U16Target::BC => run_instruction_in_register!(read_bc => u16 => write_bc, $self.$instruction),
            U16Target::DE => run_instruction_in_register!(read_de => u16 => write_de, $self.$instruction),
            U16Target::HL => run_instruction_in_register!(read_hl => u16 => write_hl, $self.$instruction),
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

macro_rules! load_in_register {
    ($input_register: ident => $main_register: ident, $self:ident) => {{
        let value = $self.registers.$input_register;
        $self.registers.$main_register = value;
        // compute next PC value
        // modulo operation to avoid overflowing effects
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! load_input_register {
    ($input_register: ident => $main_register: ident, $self:ident) => {{
        match $input_register {
            ArithmeticTarget::A => load_in_register!(a => $main_register, $self),
            ArithmeticTarget::B => load_in_register!(b => $main_register, $self),
            ArithmeticTarget::C => load_in_register!(c => $main_register, $self),
            ArithmeticTarget::D => load_in_register!(d => $main_register, $self),
            ArithmeticTarget::E => load_in_register!(e => $main_register, $self),
            ArithmeticTarget::H => load_in_register!(h => $main_register, $self),
            ArithmeticTarget::L => load_in_register!(l => $main_register, $self),
            ArithmeticTarget::HL => {
                let address = $self.registers.read_hl();
                let value = $self.bus.read_byte(address);
                $self.registers.$main_register = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            ArithmeticTarget::D8 => {
                let address = $self.pc.wrapping_add(1);
                let value = $self.bus.read_byte(address);
                $self.registers.$main_register = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(2)
            }
        }
    }};
}

macro_rules! load_in_memory {
    ($input_register: ident, $self:ident) => {{
        let address = $self.registers.read_hl();
        let value = $self.registers.$input_register;
        $self.bus.write_byte(address, value);
        // compute next PC value
        // modulo operation to avoid overflowing effects
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! load_reg_in_memory {
    ($input_register: ident, $self:ident) => {{
        match $input_register {
            ArithmeticTarget::A => load_in_memory!(a, $self),
            ArithmeticTarget::B => load_in_memory!(b, $self),
            ArithmeticTarget::C => load_in_memory!(c, $self),
            ArithmeticTarget::D => load_in_memory!(d, $self),
            ArithmeticTarget::E => load_in_memory!(e, $self),
            ArithmeticTarget::H => load_in_memory!(h, $self),
            ArithmeticTarget::L => load_in_memory!(l, $self),
            ArithmeticTarget::HL => 0,
            ArithmeticTarget::D8 => {
                let value_address = $self.pc.wrapping_add(1);
                let value = $self.bus.read_byte(value_address);
                let mem_address = $self.registers.read_hl();
                $self.bus.write_byte(mem_address, value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(2)
            }
        }
    }};
}

macro_rules! load_indirect {
    ($register: ident, $self:ident) => {{
        match $register {
            Load16Target::BC => {
                let address = $self.registers.read_bc();
                let value = $self.bus.read_byte(address);
                $self.registers.a = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::DE => {
                let address = $self.registers.read_de();
                let value = $self.bus.read_byte(address);
                $self.registers.a = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_plus => {
                let address = $self.registers.read_hl();
                let value = $self.bus.read_byte(address);
                $self.registers.a = value;
                let new_address = address.wrapping_add(1);
                $self.registers.write_hl(new_address);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_minus => {
                let address = $self.registers.read_hl();
                let value = $self.bus.read_byte(address);
                $self.registers.a = value;
                let new_address = address.wrapping_sub(1);
                $self.registers.write_hl(new_address);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
        }
    }};
}

macro_rules! store_indirect {
    ($register: ident, $self:ident) => {{
        match $register {
            Load16Target::BC => {
                let value = $self.registers.a;
                let address = $self.registers.read_bc();
                $self.bus.write_byte(address, value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::DE => {
                let value = $self.registers.a;
                let address = $self.registers.read_de();
                $self.bus.write_byte(address, value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_plus => {
                let value = $self.registers.a;
                let address = $self.registers.read_hl();
                $self.bus.write_byte(address, value);
                let new_address = address.wrapping_add(1);
                $self.registers.write_hl(new_address);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_minus => {
                let value = $self.registers.a;
                let address = $self.registers.read_hl();
                $self.bus.write_byte(address, value);
                let new_address = address.wrapping_sub(1);
                $self.registers.write_hl(new_address);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
        }
    }};
}

macro_rules! load_immediate {
    ($register: ident, $self:ident) => {{
        match $register {
            U16Target::BC => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $self.bus.read_byte(low_address);
                let high_byte = $self.bus.read_byte(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.registers.write_bc(value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
            U16Target::DE => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $self.bus.read_byte(low_address);
                let high_byte = $self.bus.read_byte(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.registers.write_de(value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
            U16Target::HL => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $self.bus.read_byte(low_address);
                let high_byte = $self.bus.read_byte(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.registers.write_hl(value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
            U16Target::SP => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $self.bus.read_byte(low_address);
                let high_byte = $self.bus.read_byte(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.sp = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
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
            // Arithmetic instructions
            Instruction::ADD(target) => arithmetic_instruction!(target, self.add),
            Instruction::ADD16(target) => arithmetic_instruction!(target => u16 => self.add16),
            Instruction::ADDC(target) => arithmetic_instruction!(target, self.addc),
            Instruction::SUB(target) => arithmetic_instruction!(target, self.sub),
            Instruction::SBC(target) => arithmetic_instruction!(target, self.subc),
            Instruction::AND(target) => arithmetic_instruction!(target, self.and),
            Instruction::XOR(target) => arithmetic_instruction!(target, self.xor),
            Instruction::OR(target) => arithmetic_instruction!(target, self.or),
            Instruction::CP(target) => arithmetic_instruction!(target, self.cp),

            // Increment & decrement instructions
            Instruction::INC(target) => inc_dec_instruction!(target, self.inc),
            Instruction::INC16(target) => inc_dec_instruction!(target => u16 => self.inc16),
            Instruction::DEC(target) => inc_dec_instruction!(target, self.dec),
            Instruction::DEC16(target) => inc_dec_instruction!(target => u16 => self.dec16),

            // Load & Store instructions
            Instruction::LOAD(main_reg, input_reg) => self.load(input_reg, main_reg),
            Instruction::LOAD_INDIRECT(target) => load_indirect!(target, self),
            Instruction::LOAD_IMMEDIATE(target) => load_immediate!(target, self),
            Instruction::STORE_INDIRECT(target) => store_indirect!(target, self),

            // Jump instructions
            Instruction::JUMP_RELATIVE(target) => self.jump_relative(target),
            Instruction::JUMP_IMMEDIATE(target) => 0,
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

    fn load(&mut self, input_register: ArithmeticTarget, main_register: IncDecTarget) -> u16 {
        match main_register {
            IncDecTarget::A => load_input_register!(input_register => a, self),
            IncDecTarget::B => load_input_register!(input_register => b, self),
            IncDecTarget::C => load_input_register!(input_register => c, self),
            IncDecTarget::D => load_input_register!(input_register => d, self),
            IncDecTarget::E => load_input_register!(input_register => e, self),
            IncDecTarget::H => load_input_register!(input_register => h, self),
            IncDecTarget::L => load_input_register!(input_register => l, self),
            IncDecTarget::HL => load_reg_in_memory!(input_register, self),
        }
    }

    fn jump_relative(&mut self, target: JumpRelativeTarget) -> u16 {
        match target {
            JumpRelativeTarget::NZ => {
                let flag_value = self.registers.f.zero;
                let immediate_address = self.pc.wrapping_add(1);
                let immediate = self.bus.read_byte(immediate_address);
                if !flag_value {
                    self.pc.wrapping_add(immediate as u16)
                } else {
                    self.pc.wrapping_add(2)
                }
            }
            JumpRelativeTarget::NC => {
                let flag_value = self.registers.f.carry;
                let immediate_address = self.pc.wrapping_add(1);
                let immediate = self.bus.read_byte(immediate_address);
                if !flag_value {
                    self.pc.wrapping_add(immediate as u16)
                } else {
                    self.pc.wrapping_add(2)
                }
            }
            JumpRelativeTarget::IMMEDIATE => {
                let immediate_address = self.pc.wrapping_add(1);
                let immediate = self.bus.read_byte(immediate_address);
                self.pc.wrapping_add(immediate as u16)
            }
            JumpRelativeTarget::Z => {
                let flag_value = self.registers.f.zero;
                let immediate_address = self.pc.wrapping_add(1);
                let immediate = self.bus.read_byte(immediate_address);
                if flag_value {
                    self.pc.wrapping_add(immediate as u16)
                } else {
                    self.pc.wrapping_add(2)
                }
            }
            JumpRelativeTarget::C => {
                let flag_value = self.registers.f.carry;
                let immediate_address = self.pc.wrapping_add(1);
                let immediate = self.bus.read_byte(immediate_address);
                if flag_value {
                    self.pc.wrapping_add(immediate as u16)
                } else {
                    self.pc.wrapping_add(2)
                }
            }
        }
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::*;
    use crate::cpu::instruction::ArithmeticTarget::{B, C, D, D8, E, H, HL};
    use crate::cpu::instruction::Instruction::{
        ADD, ADD16, ADDC, AND, CP, DEC, DEC16, INC, INC16, LOAD, LOAD_IMMEDIATE, LOAD_INDIRECT, OR,
        SBC, STORE_INDIRECT, SUB, XOR,
    };
    use crate::cpu::instruction::{IncDecTarget, Load16Target, U16Target};

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

    #[test]
    fn test_load_registers() {
        let mut cpu = Cpu::new();

        cpu.registers.write_de(0x0057);
        cpu.execute(LOAD(IncDecTarget::B, E));
        assert_eq!(cpu.registers.read_bc(), 0x5700);

        cpu.registers.write_hl(0x6400);
        cpu.execute(LOAD(IncDecTarget::C, H));
        assert_eq!(cpu.registers.read_bc(), 0x5764);

        let mut mem_address = 0x0001;
        let mut data = 0x23;
        cpu.bus.write_byte(mem_address, data);
        cpu.execute(LOAD(IncDecTarget::A, D8));
        assert_eq!(cpu.registers.read_af(), 0x2300);

        mem_address = 0x0010;
        cpu.registers.write_hl(mem_address);
        cpu.execute(LOAD(IncDecTarget::HL, D8));
        assert_eq!(cpu.bus.read_byte(mem_address), 0x23);

        mem_address = 0x002A;
        cpu.registers.write_hl(mem_address);
        cpu.registers.write_de(0xD500);
        cpu.execute(LOAD(IncDecTarget::HL, D));
        assert_eq!(cpu.bus.read_byte(mem_address), 0xD5);

        mem_address = 0x00C8;
        data = 0x5F;
        cpu.bus.write_byte(mem_address, data);
        cpu.registers.write_hl(mem_address);
        cpu.execute(LOAD(IncDecTarget::A, HL));
        assert_eq!(cpu.registers.read_af(), 0x5F00);

        cpu.registers.write_de(0x0000);
        cpu.execute(LOAD(IncDecTarget::E, HL));
        assert_eq!(cpu.registers.read_de(), 0x005F);
    }

    #[test]
    fn test_load_indirect() {
        let mut cpu = Cpu::new();

        let mem_address = 0x00D8;
        let mut data = 0x56;
        cpu.bus.write_byte(mem_address, data);
        cpu.registers.write_bc(mem_address);
        cpu.execute(LOAD_INDIRECT(Load16Target::BC));
        assert_eq!(cpu.registers.read_af(), 0x5600);

        data = 0xC6;
        cpu.bus.write_byte(mem_address, data);
        cpu.registers.write_hl(mem_address);
        cpu.execute(LOAD_INDIRECT(Load16Target::HL_plus));
        assert_eq!(cpu.registers.read_af(), 0xC600);
        assert_eq!(cpu.registers.read_hl(), 0x00D9);
    }

    #[test]
    fn test_load_immediate() {
        let mut cpu = Cpu::new();

        let low_data = 0x4C;
        let high_data = 0xB7;
        let value = ((high_data as u16) << 8) + low_data as u16;
        cpu.bus.write_byte(0x0001, low_data);
        cpu.bus.write_byte(0x0002, high_data);
        cpu.execute(LOAD_IMMEDIATE(U16Target::DE));
        assert_eq!(cpu.registers.read_de(), value);

        cpu.execute(LOAD_IMMEDIATE(U16Target::SP));
        assert_eq!(cpu.sp, value);
    }

    #[test]
    fn test_store_indirect() {
        let mut cpu = Cpu::new();

        let mem_address = 0x00D8;
        let mut data = 0x5600;
        cpu.registers.write_af(data);
        cpu.registers.write_de(mem_address);
        cpu.execute(STORE_INDIRECT(Load16Target::DE));
        assert_eq!(cpu.bus.read_byte(mem_address), 0x56);

        data = 0xC600;
        cpu.registers.write_af(data);
        cpu.registers.write_hl(mem_address);
        cpu.execute(STORE_INDIRECT(Load16Target::HL_minus));
        assert_eq!(cpu.bus.read_byte(mem_address), 0xC6);
        assert_eq!(cpu.registers.read_hl(), 0x00D7);
    }

    #[test]
    fn test_jump_relative_nzero() {
        let mut cpu = Cpu::new();

        // first, fill memory with program
        let base_address: u16 = 0x0000;
        let jump: u8 = 0x05;
        let jump_nz: u8 = 0x20;
        let program: [u8; 10] = [
            jump_nz, jump, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        let mut index = 0;
        for data in program {
            cpu.bus.write_byte(base_address + index, data);
            index += 1;
        }

        // run CPU to do the jump
        cpu.run();
        assert_eq!(
            cpu.bus.read_byte(cpu.pc),
            cpu.bus.read_byte(base_address + (jump as u16))
        );

        // reset CPU and run it with the flag, we don't do the jump
        cpu.registers.f.zero = true;
        cpu.pc = base_address;
        cpu.run();
        assert_eq!(
            cpu.bus.read_byte(cpu.pc),
            cpu.bus.read_byte(base_address + 2)
        );
    }

    #[test]
    fn test_jump_relative_carry() {
        let mut cpu = Cpu::new();

        // first, fill memory with program
        let base_address: u16 = 0x0000;
        let jump: u8 = 0x05;
        let jump_carry: u8 = 0x38;
        let program: [u8; 10] = [
            jump_carry, jump, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        let mut index = 0;
        for data in program {
            cpu.bus.write_byte(base_address + index, data);
            index += 1;
        }

        // run CPU to do the jump
        cpu.registers.f.carry = true;
        cpu.run();
        assert_eq!(
            cpu.bus.read_byte(cpu.pc),
            cpu.bus.read_byte(base_address + (jump as u16))
        );

        // reset CPU and run it with the flag, we don't do the jump
        cpu.registers.f.carry = false;
        cpu.pc = base_address;
        cpu.run();
        assert_eq!(
            cpu.bus.read_byte(cpu.pc),
            cpu.bus.read_byte(base_address + 2)
        );
    }
}
