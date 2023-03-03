mod instruction;
mod register;

use instruction::{
    ArithmeticTarget, BitTarget, Direction, IncDecTarget, Instruction, JumpTarget, Load16Target,
    PopPushTarget, RamTarget, ResetTarget, SPTarget, U16Target,
};
use register::Registers;

use crate::soc::peripheral::{Peripheral, VBLANK_VECTOR, LCDSTAT_VECTOR, TIMER_VECTOR};
use crate::soc::peripheral::nvic::InterruptSources;

const RUN_0_CYCLE: u8 = 0;
const RUN_1_CYCLE: u8 = 1;
const RUN_2_CYCLES: u8 = 2;
const RUN_3_CYCLES: u8 = 3;
const RUN_4_CYCLES: u8 = 4;
const RUN_5_CYCLES: u8 = 5;
const RUN_6_CYCLES: u8 = 6;

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
    ($target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $target {
            ArithmeticTarget::A => (run_instruction_in_register!(a => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::B => (run_instruction_in_register!(b => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::C => (run_instruction_in_register!(c => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::D => (run_instruction_in_register!(d => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::E => (run_instruction_in_register!(e => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::H => (run_instruction_in_register!(h => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::L => (run_instruction_in_register!(l => a, $self.$instruction), RUN_1_CYCLE),
            ArithmeticTarget::HL => ({
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                let new_value = $self.$instruction(value);
                $self.registers.a = new_value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }, RUN_2_CYCLES),
            ArithmeticTarget::D8 => ({
                let address = $self.pc.wrapping_add(1);
                let value = $peripheral.read(address);
                let new_value = $self.$instruction(value);
                $self.registers.a = new_value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(2)
            }, RUN_2_CYCLES),
        }
    }};

    ($target: ident => $flag:ident => $self:ident.$instruction:ident) => {{
        match $target {
            U16Target::BC => (run_instruction_in_register!(read_bc => u16 => write_hl, $self.$instruction), RUN_2_CYCLES),
            U16Target::DE => (run_instruction_in_register!(read_de => u16 => write_hl, $self.$instruction), RUN_2_CYCLES),
            U16Target::HL => (run_instruction_in_register!(read_hl => u16 => write_hl, $self.$instruction), RUN_2_CYCLES),
            U16Target::SP => ({
                let value_in_register = $self.sp;
                let new_value = $self.$instruction(value_in_register);
                $self.registers.write_hl(new_value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }, RUN_2_CYCLES),
        }
    }};
}

macro_rules! inc_dec_instruction {
    ($target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $target {
            IncDecTarget::A => (run_instruction_in_register!(a => a, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::B => (run_instruction_in_register!(b => b, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::C => (run_instruction_in_register!(c => c, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::D => (run_instruction_in_register!(d => d, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::E => (run_instruction_in_register!(e => e, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::H => (run_instruction_in_register!(h => h, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::L => (run_instruction_in_register!(l => l, $self.$instruction), RUN_1_CYCLE),
            IncDecTarget::HL => ({
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                let new_value = $self.$instruction(value);
                $peripheral.write(address, new_value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }, RUN_3_CYCLES),
        }
    }};

    ($target: ident => $flag:ident => $self:ident.$instruction:ident) => {{
        match $target {
            U16Target::BC => (run_instruction_in_register!(read_bc => u16 => write_bc, $self.$instruction), RUN_2_CYCLES),
            U16Target::DE => (run_instruction_in_register!(read_de => u16 => write_de, $self.$instruction), RUN_2_CYCLES),
            U16Target::HL => (run_instruction_in_register!(read_hl => u16 => write_hl, $self.$instruction), RUN_2_CYCLES),
            U16Target::SP => ({
                let value_in_register = $self.sp;
                let new_value = $self.$instruction(value_in_register);
                $self.sp = new_value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }, RUN_2_CYCLES),
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
    ($input_register: ident => $main_register: ident, $self:ident, $peripheral:expr) => {{
        match $input_register {
            ArithmeticTarget::A => (load_in_register!(a => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::B => (load_in_register!(b => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::C => (load_in_register!(c => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::D => (load_in_register!(d => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::E => (load_in_register!(e => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::H => (load_in_register!(h => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::L => (load_in_register!(l => $main_register, $self), RUN_1_CYCLE),
            ArithmeticTarget::HL => ({
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                $self.registers.$main_register = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }, RUN_2_CYCLES),
            ArithmeticTarget::D8 => ({
                let address = $self.pc.wrapping_add(1);
                let value = $peripheral.read(address);
                $self.registers.$main_register = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(2)
            }, RUN_2_CYCLES),
        }
    }};
}

macro_rules! load_in_memory {
    ($input_register: ident, $self:ident, $peripheral:expr) => {{
        let address = $self.registers.read_hl();
        let value = $self.registers.$input_register;
        $peripheral.write(address, value);
        // compute next PC value
        // modulo operation to avoid overflowing effects
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! load_reg_in_memory {
    ($input_register: ident, $self:ident, $peripheral:expr) => {{
        match $input_register {
            ArithmeticTarget::A => (load_in_memory!(a, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::B => (load_in_memory!(b, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::C => (load_in_memory!(c, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::D => (load_in_memory!(d, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::E => (load_in_memory!(e, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::H => (load_in_memory!(h, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::L => (load_in_memory!(l, $self, $peripheral), RUN_2_CYCLES),
            ArithmeticTarget::HL => (0, RUN_0_CYCLE),
            ArithmeticTarget::D8 => ({
                let value_address = $self.pc.wrapping_add(1);
                let value = $peripheral.read(value_address);
                let mem_address = $self.registers.read_hl();
                $peripheral.write(mem_address, value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(2)
            }, RUN_3_CYCLES),
        }
    }};
}

macro_rules! load_indirect {
    ($register: ident, $self:ident, $peripheral:expr) => {{
        match $register {
            Load16Target::BC => {
                let address = $self.registers.read_bc();
                let value = $peripheral.read(address);
                $self.registers.a = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::DE => {
                let address = $self.registers.read_de();
                let value = $peripheral.read(address);
                $self.registers.a = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_plus => {
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                $self.registers.a = value;
                let new_address = address.wrapping_add(1);
                $self.registers.write_hl(new_address);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_minus => {
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
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
    ($register: ident, $self:ident, $peripheral:expr) => {{
        match $register {
            Load16Target::BC => {
                let value = $self.registers.a;
                let address = $self.registers.read_bc();
                $peripheral.write(address, value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::DE => {
                let value = $self.registers.a;
                let address = $self.registers.read_de();
                $peripheral.write(address, value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_plus => {
                let value = $self.registers.a;
                let address = $self.registers.read_hl();
                $peripheral.write(address, value);
                let new_address = address.wrapping_add(1);
                $self.registers.write_hl(new_address);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(1)
            }
            Load16Target::HL_minus => {
                let value = $self.registers.a;
                let address = $self.registers.read_hl();
                $peripheral.write(address, value);
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
    ($register: ident, $self:ident, $peripheral:expr) => {{
        match $register {
            U16Target::BC => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $peripheral.read(low_address);
                let high_byte = $peripheral.read(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.registers.write_bc(value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
            U16Target::DE => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $peripheral.read(low_address);
                let high_byte = $peripheral.read(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.registers.write_de(value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
            U16Target::HL => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $peripheral.read(low_address);
                let high_byte = $peripheral.read(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.registers.write_hl(value);
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
            U16Target::SP => {
                let low_address = $self.pc.wrapping_add(1);
                let high_address = $self.pc.wrapping_add(2);
                let low_byte = $peripheral.read(low_address);
                let high_byte = $peripheral.read(high_address);
                let value = (low_byte as u16) + ((high_byte as u16) << 8);
                $self.sp = value;
                // compute next PC value
                // modulo operation to avoid overflowing effects
                $self.pc.wrapping_add(3)
            }
        }
    }};
}

macro_rules! control_with_flag {
    ($negative: ident, $self:ident.$instruction:ident, $flag:ident, $peripheral:expr) => {{
        let flag_value = $self.registers.f.$flag;
        // inverse flag if negative option is selected
        let mut new_flag = flag_value;
        if $negative {
            new_flag = !flag_value;
        }
        // execute instruction
        $self.$instruction(new_flag, $peripheral)
    }};
}

macro_rules! control {
    ($flag: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $flag {
            JumpTarget::NZ => control_with_flag!(true, $self.$instruction, zero, $peripheral),
            JumpTarget::NC => control_with_flag!(true, $self.$instruction, carry, $peripheral),
            JumpTarget::Z => control_with_flag!(false, $self.$instruction, zero, $peripheral),
            JumpTarget::C => control_with_flag!(false, $self.$instruction, carry, $peripheral),
            JumpTarget::IMMEDIATE => $self.$instruction(true, $peripheral),
        }
    }};
}

macro_rules! pop {
    ($target:ident, $self:ident, $peripheral:expr) => {{
        match $target {
            PopPushTarget::BC => {
                let pop_data = $self.pop($peripheral);
                $self.registers.write_bc(pop_data);

                // return next pc
                $self.pc.wrapping_add(1)
            }
            PopPushTarget::DE => {
                let pop_data = $self.pop($peripheral);
                $self.registers.write_de(pop_data);

                // return next pc
                $self.pc.wrapping_add(1)
            }
            PopPushTarget::HL => {
                let pop_data = $self.pop($peripheral);
                $self.registers.write_hl(pop_data);

                // return next pc
                $self.pc.wrapping_add(1)
            }
            PopPushTarget::AF => {
                let pop_data = $self.pop($peripheral);
                $self.registers.write_af(pop_data);

                // return next pc
                $self.pc.wrapping_add(1)
            }
        }
    }};
}

macro_rules! push {
    ($target: ident, $self:ident, $peripheral:expr) => {{
        match $target {
            PopPushTarget::BC => {
                let push_data = $self.registers.read_bc();
                $self.push(push_data, $peripheral);

                // return next pc
                $self.pc.wrapping_add(1)
            }
            PopPushTarget::DE => {
                let push_data = $self.registers.read_de();
                $self.push(push_data, $peripheral);

                // return next pc
                $self.pc.wrapping_add(1)
            }
            PopPushTarget::HL => {
                let push_data = $self.registers.read_hl();
                $self.push(push_data, $peripheral);

                // return next pc
                $self.pc.wrapping_add(1)
            }
            PopPushTarget::AF => {
                let push_data = $self.registers.read_af();
                $self.push(push_data, $peripheral);

                // return next pc
                $self.pc.wrapping_add(1)
            }
        }
    }};
}

macro_rules! ret {
    ($target: ident, $self:ident, $peripheral:expr) => {{
        match $target {
            JumpTarget::NZ => {
                if !$self.registers.f.zero {
                    ($self.pop($peripheral), RUN_5_CYCLES)
                } else {
                    ($self.pc.wrapping_add(1), RUN_2_CYCLES)
                }
            }
            JumpTarget::NC => {
                if !$self.registers.f.carry {
                    ($self.pop($peripheral), RUN_5_CYCLES)
                } else {
                    ($self.pc.wrapping_add(1), RUN_2_CYCLES)
                }
            }
            JumpTarget::Z => {
                if $self.registers.f.zero {
                    ($self.pop($peripheral), RUN_5_CYCLES)
                } else {
                    ($self.pc.wrapping_add(1), RUN_2_CYCLES)
                }
            }
            JumpTarget::C => {
                if $self.registers.f.carry {
                    ($self.pop($peripheral), RUN_5_CYCLES)
                } else {
                    ($self.pc.wrapping_add(1), RUN_2_CYCLES)
                }
            }
            JumpTarget::IMMEDIATE => ($self.pop($peripheral), RUN_4_CYCLES),
        }
    }};
}

macro_rules! reset {
    ($target: ident, $self:ident, $peripheral:expr) => {{
        match $target {
            ResetTarget::FLASH_0 => ($self.reset(0x00, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_1 => ($self.reset(0x08, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_2 => ($self.reset(0x10, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_3 => ($self.reset(0x18, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_4 => ($self.reset(0x20, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_5 => ($self.reset(0x28, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_6 => ($self.reset(0x30, $peripheral), RUN_4_CYCLES),
            ResetTarget::FLASH_7 => ($self.reset(0x38, $peripheral), RUN_4_CYCLES),
        }
    }};
}

macro_rules! interrupt_enable {
    ($enable: ident, $self:ident, $peripheral:expr) => {{
        $peripheral.nvic.master_enable($enable);
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! rotate_register {
    ($register: ident, $self:ident.$instruction:ident, $direction: ident, $zero:ident) => {{
        // update flag register
        $self.registers.f.substraction = false;
        $self.registers.f.half_carry = false;
        // rotate register
        let new_value = $self.$instruction($self.registers.$register, $direction, false);
        $self.registers.$register = new_value;
        // return next pc
        $self.pc.wrapping_add(1)
    }};
}

macro_rules! rotate_from_register {
    ($target: ident, $self:ident.$instruction:ident, $direction: ident, $peripheral:expr) => {{
        match $target {
            IncDecTarget::A => {
                let next_pc = rotate_register!(a, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::B => {
                let next_pc = rotate_register!(b, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::C => {
                let next_pc = rotate_register!(c, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::D => {
                let next_pc = rotate_register!(d, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::E => {
                let next_pc = rotate_register!(e, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::H => {
                let next_pc = rotate_register!(h, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::L => {
                let next_pc = rotate_register!(l, $self.$instruction, $direction, true);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::HL => {
                // update flag register
                $self.registers.f.substraction = false;
                $self.registers.f.half_carry = false;
                // get data from memory
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                // rotate value
                let new_value = $self.$instruction(value, $direction, true);
                // save value in memory
                $peripheral.write(address, new_value);
                // return next pc
                ($self.pc.wrapping_add(2), RUN_4_CYCLES)
            }
        }
    }};
}

macro_rules! shift {
    ($target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $target {
            IncDecTarget::A => {
                let next_pc = run_instruction_in_register!(a => a, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::B => {
                let next_pc = run_instruction_in_register!(b => b, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::C => {
                let next_pc = run_instruction_in_register!(c => c, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::D => {
                let next_pc = run_instruction_in_register!(d => d, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::E => {
                let next_pc = run_instruction_in_register!(e => e, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::H => {
                let next_pc = run_instruction_in_register!(h => h, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::L => {
                let next_pc = run_instruction_in_register!(l => l, $self.$instruction);
                (next_pc.wrapping_add(1), RUN_2_CYCLES)
            }
            IncDecTarget::HL => {
                // get data from memory
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                // rotate value
                let new_value = $self.$instruction(value);
                // save value in memory
                $peripheral.write(address, new_value);
                // return next pc
                ($self.pc.wrapping_add(2), RUN_4_CYCLES)
            }
        }
    }};
}

macro_rules! long_inst_from_reg {
    ($bit: expr, $target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $target {
            IncDecTarget::A => ($self.$instruction($bit, $self.registers.a), RUN_2_CYCLES),
            IncDecTarget::B => ($self.$instruction($bit, $self.registers.b), RUN_2_CYCLES),
            IncDecTarget::C => ($self.$instruction($bit, $self.registers.c), RUN_2_CYCLES),
            IncDecTarget::D => ($self.$instruction($bit, $self.registers.d), RUN_2_CYCLES),
            IncDecTarget::E => ($self.$instruction($bit, $self.registers.e), RUN_2_CYCLES),
            IncDecTarget::H => ($self.$instruction($bit, $self.registers.h), RUN_2_CYCLES),
            IncDecTarget::L => ($self.$instruction($bit, $self.registers.l), RUN_2_CYCLES),
            IncDecTarget::HL => ({
                // get data from memory
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                // complement value
                $self.$instruction($bit, value);
                // return next pc
                $self.pc.wrapping_add(2)
            },  RUN_4_CYCLES),
        }
    }};

    ($enable: ident, $bit: expr => $target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $target {
            IncDecTarget::A => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.a);
                $self.registers.a = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::B => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.b);
                $self.registers.b = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::C => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.c);
                $self.registers.c = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::D => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.d);
                $self.registers.d = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::E => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.e);
                $self.registers.e = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::H => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.h);
                $self.registers.h = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::L => {
                let new_value = $self.$instruction($enable, $bit, $self.registers.l);
                $self.registers.l = new_value;
                // return next_pc
                ($self.pc.wrapping_add(2), RUN_2_CYCLES)
            }
            IncDecTarget::HL => {
                // get data from memory
                let address = $self.registers.read_hl();
                let value = $peripheral.read(address);
                // run instruction on value
                let new_value = $self.$instruction($enable, $bit, value);
                // save new value in memory
                $peripheral.write(address, new_value);
                // return next pc
                ($self.pc.wrapping_add(2), RUN_4_CYCLES)
            }
        }
    }};
}

macro_rules! long_inst {
    ($bit: ident, $target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $bit {
            BitTarget::BIT_0 => long_inst_from_reg!(0, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_1 => long_inst_from_reg!(1, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_2 => long_inst_from_reg!(2, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_3 => long_inst_from_reg!(3, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_4 => long_inst_from_reg!(4, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_5 => long_inst_from_reg!(5, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_6 => long_inst_from_reg!(6, $target, $self.$instruction, $peripheral),
            BitTarget::BIT_7 => long_inst_from_reg!(7, $target, $self.$instruction, $peripheral),
        }
    }};

    ($enable: ident, $bit: ident => $target: ident, $self:ident.$instruction:ident, $peripheral:expr) => {{
        match $bit {
            BitTarget::BIT_0 => long_inst_from_reg!($enable, 0 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_1 => long_inst_from_reg!($enable, 1 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_2 => long_inst_from_reg!($enable, 2 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_3 => long_inst_from_reg!($enable, 3 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_4 => long_inst_from_reg!($enable, 4 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_5 => long_inst_from_reg!($enable, 5 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_6 => long_inst_from_reg!($enable, 6 => $target, $self.$instruction, $peripheral),
            BitTarget::BIT_7 => long_inst_from_reg!($enable, 7 => $target, $self.$instruction, $peripheral),
        }
    }};
}

#[derive(PartialEq)]
pub enum CpuMode {
    RUN,
    INTERRUPT,
    STOP,
    HALT,
}

pub enum CarryOp {
    SET,
    FLIP,
}

pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    mode: CpuMode,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            pc: 0x0000,
            sp: 0x0000,
            mode: CpuMode::RUN,
        }
    }

    fn decode(&mut self, instruction_byte: u8, peripheral: &mut Peripheral) -> Option<Instruction> {
        if Instruction::is_long_instruction(instruction_byte) {
            let long_instruction_byte = peripheral.read(self.pc.wrapping_add(1));
            Instruction::from_long_byte(long_instruction_byte)
        } else {
            Instruction::from_byte(instruction_byte)
        }
    }

    pub fn run(&mut self, peripheral: &mut Peripheral) -> u8 {
        // catch interrupt as soon as possible
        if peripheral.nvic.is_an_interrupt_to_run() {
            self.mode = CpuMode::INTERRUPT;
        }
    
        // run CPU if it's not in HALT or STOP mode
        match self.mode {
    
            CpuMode::RUN => {
                // fetch instruction
                let instruction_byte = peripheral.read(self.pc);
                // decode instruction
                let (next_pc, cpu_cycles) = if let Some(instruction) = self.decode(instruction_byte, peripheral) {
                    // execute instruction
                    self.execute(instruction, peripheral)
                } else {
                    panic!("Unknown instruction found for 0x{:x}", instruction_byte);
                };

                // update PC value & cycles value
                self.pc = next_pc;
    
                // run the peripheral subsystem
                cpu_cycles
            }
    
            CpuMode::INTERRUPT => {
                // get the interrupt source
                if let Some(interrupt_source) = peripheral.nvic.get_interrupt() {
                    // set the cpu in RUN mode to handle interrupt routine
                    self.mode = CpuMode::RUN;
                    // disable interrupts while handling interrupt routine
                    peripheral.nvic.master_enable(false);
                    // jump to interrupt routine
                    self.jump_to_interrupt_routine(interrupt_source, peripheral);
                    // 2 NOP (2 cycles) + PUSH (2 cycles) + set PC (1 cycle)

                    // run the peripheral subsystem
                    RUN_5_CYCLES
                } else {
                    panic!("An interrupt has been triggered but no interrupt source has been found !")
                }
            }
    
            CpuMode::HALT => {
                // exit HALT mode if an interrupt is pending
                if peripheral.nvic.is_an_interrupt_pending() {
                    self.mode = CpuMode::RUN
                }
    
                // oscillator and LCD controller are not stopped in HALT mode
                // run the peripheral subsystem
                RUN_1_CYCLE
            }
    
            CpuMode::STOP => {
                // all system is stopped
                RUN_0_CYCLE
            }
        }
    }

    fn jump_to_interrupt_routine(&mut self, interrupt_source: InterruptSources, peripheral: &mut Peripheral) {
        self.push(self.pc, peripheral);
        match interrupt_source {
            InterruptSources::VBLANK => self.pc = VBLANK_VECTOR,
            InterruptSources::STAT => self.pc = LCDSTAT_VECTOR,
            InterruptSources::TIMER => self.pc = TIMER_VECTOR,
            _ => {},
        }
    }

    fn execute(&mut self, instruction: Instruction, peripheral: &mut Peripheral) -> (u16, u8) {
        match instruction {
            // Arithmetic instructions
            Instruction::ADD(target) => arithmetic_instruction!(target, self.add, peripheral),
            Instruction::ADD16(target) => arithmetic_instruction!(target => u16 => self.add16),
            Instruction::ADDC(target) => arithmetic_instruction!(target, self.addc, peripheral),
            Instruction::SUB(target) => arithmetic_instruction!(target, self.sub, peripheral),
            Instruction::SBC(target) => arithmetic_instruction!(target, self.subc, peripheral),
            Instruction::AND(target) => arithmetic_instruction!(target, self.and, peripheral),
            Instruction::XOR(target) => arithmetic_instruction!(target, self.xor, peripheral),
            Instruction::OR(target) => arithmetic_instruction!(target, self.or, peripheral),
            Instruction::CP(target) => arithmetic_instruction!(target, self.cp, peripheral),
            Instruction::AddSp => (self.add_sp(peripheral), 4),

            // Increment & decrement instructions
            Instruction::INC(target) => inc_dec_instruction!(target, self.inc, peripheral),
            Instruction::INC16(target) => inc_dec_instruction!(target => u16 => self.inc16),
            Instruction::DEC(target) => inc_dec_instruction!(target, self.dec, peripheral),
            Instruction::DEC16(target) => inc_dec_instruction!(target => u16 => self.dec16),

            // Load & Store instructions
            Instruction::LOAD(main_reg, input_reg) => self.load(input_reg, main_reg, peripheral),
            Instruction::LOAD_INDIRECT(target) => (load_indirect!(target, self, peripheral), RUN_2_CYCLES),
            Instruction::LOAD_IMMEDIATE(target) => (load_immediate!(target, self, peripheral), RUN_3_CYCLES),
            Instruction::STORE_INDIRECT(target) => (store_indirect!(target, self, peripheral), RUN_2_CYCLES),
            Instruction::LOAD_SP(target) => self.load_sp(target, peripheral),
            Instruction::LOAD_RAM(target) => self.load_store_ram(target, true, peripheral),
            Instruction::STORE_RAM(target) => self.load_store_ram(target, false, peripheral),

            // JUMP / CALL / RETURN / RESET instructions
            Instruction::JUMP_RELATIVE(target) => control!(target, self.jump_relative, peripheral),
            Instruction::JUMP_IMMEDIATE(target) => control!(target, self.jump_immediate, peripheral),
            Instruction::JUMP_INDIRECT => (self.jump_indirect(), RUN_1_CYCLE),
            Instruction::RETURN(target) => ret!(target, self, peripheral),
            Instruction::RESET(target) => reset!(target, self, peripheral),
            Instruction::CALL(target) => control!(target, self.call, peripheral),
            Instruction::RETI => (self.reti(peripheral), RUN_4_CYCLES),

            // Pop & Push instructions
            Instruction::POP(target) => (pop!(target, self, peripheral), RUN_3_CYCLES),
            Instruction::PUSH(target) => (push!(target, self, peripheral), RUN_4_CYCLES),

            // Interrupt instructions
            Instruction::DI => (interrupt_enable!(false, self, peripheral), RUN_1_CYCLE),
            Instruction::EI => (interrupt_enable!(true, self, peripheral), RUN_1_CYCLE),

            // Control instructions
            Instruction::NOP => (self.pc.wrapping_add(1), RUN_1_CYCLE),
            Instruction::STOP => (self.set_cpu_mode(CpuMode::STOP), RUN_1_CYCLE),
            Instruction::HALT => (self.set_cpu_mode(CpuMode::HALT), RUN_1_CYCLE),
            Instruction::DAA => (self.decimal_adjust(), RUN_1_CYCLE),
            Instruction::SCF => (self.set_carry(CarryOp::SET), RUN_1_CYCLE),
            Instruction::CPL => (self.flip_register_a(), RUN_1_CYCLE),
            Instruction::CCF => (self.set_carry(CarryOp::FLIP), RUN_1_CYCLE),

            // Rotate, Shift & Swap instructions
            Instruction::RCA(direction) => (rotate_register!(a, self.rotate, direction, false), RUN_1_CYCLE),
            Instruction::RA(direction) => (rotate_register!(a, self.rotate_through_carry, direction, false), RUN_1_CYCLE),
            Instruction::RC(direction, target) => rotate_from_register!(target, self.rotate, direction, peripheral),
            Instruction::R(direction, target) => rotate_from_register!(target, self.rotate_through_carry, direction, peripheral),
            Instruction::SLA(target) => shift!(target, self.shift_left_and_reset, peripheral),
            Instruction::SRA(target) => shift!(target, self.shift_right, peripheral),
            Instruction::SRL(target) => shift!(target, self.shift_right_and_reset, peripheral),
            Instruction::SWAP(target) => shift!(target, self.swap, peripheral),

            // Bit instructions
            Instruction::BIT(bit, target) => long_inst!(bit, target, self.complement_bit, peripheral),
            Instruction::RESET_BIT(bit, target) => long_inst!(false, bit => target, self.set_bit, peripheral),
            Instruction::SET_BIT(bit, target) => long_inst!(true, bit => target, self.set_bit, peripheral),
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
        let (intermediate_value, first_overflow) = self.registers.a.overflowing_sub(value);
        let (new_value, second_overflow) = intermediate_value.overflowing_sub(carry);
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

    fn load(&mut self, input_register: ArithmeticTarget, main_register: IncDecTarget, peripheral: &mut Peripheral) -> (u16, u8) {
        match main_register {
            IncDecTarget::A => load_input_register!(input_register => a, self, peripheral),
            IncDecTarget::B => load_input_register!(input_register => b, self, peripheral),
            IncDecTarget::C => load_input_register!(input_register => c, self, peripheral),
            IncDecTarget::D => load_input_register!(input_register => d, self, peripheral),
            IncDecTarget::E => load_input_register!(input_register => e, self, peripheral),
            IncDecTarget::H => load_input_register!(input_register => h, self, peripheral),
            IncDecTarget::L => load_input_register!(input_register => l, self, peripheral),
            IncDecTarget::HL => load_reg_in_memory!(input_register, self, peripheral),
        }
    }

    fn load_sp(&mut self, target: SPTarget, peripheral: &mut Peripheral) -> (u16, u8) {
        match target {
            SPTarget::FROM_SP => ({
                let low_byte_address = peripheral.read(self.pc.wrapping_add(1)) as u16;
                let high_byte_address = peripheral.read(self.pc.wrapping_add(2)) as u16;
                let address = low_byte_address + (high_byte_address << 8);

                // save Stack Pointer lower byte
                let mut data = (self.sp & 0x00FF) as u8;
                peripheral.write(address, data);
                // save Stack Pointer higher byte
                data = ((self.sp & 0xFF00) >> 8) as u8;
                peripheral.write(address + 1, data);

                // return next program counter value
                self.pc.wrapping_add(3)
            }, RUN_4_CYCLES),
            SPTarget::TO_HL => ({
                let immediate = peripheral.read(self.pc.wrapping_add(1)) as i8;
                let stack_addr = if immediate >= 0 {
                    self.sp.wrapping_add(immediate as u16)
                } else {
                    // using wrapping_sub() implies to convert immediate to absolute value
                    self.sp.wrapping_sub(immediate.abs() as u16)
                };
                self.registers.write_hl(stack_addr);

                // update flags
                self.registers.f.zero = false;
                self.registers.f.substraction = false;
                self.registers.f.half_carry = (self.sp & 0xF) + (immediate as i8 as i16 as u16 & 0xF) > 0xF;
                self.registers.f.carry = (self.sp & 0xFF) + (immediate as i8 as i16 as u16 & 0xFF) > 0xFF;

                // return next program counter value
                self.pc.wrapping_add(2)
            }, RUN_3_CYCLES),
            SPTarget::TO_SP => ({
                let value = self.registers.read_hl();
                self.pc = value;

                // return next program counter value
                self.pc.wrapping_add(1)
            }, RUN_2_CYCLES),
        }
    }

    fn load_store_ram(&mut self, target: RamTarget, load: bool, peripheral: &mut Peripheral) -> (u16, u8) {
        match target {
            RamTarget::OneByteAddress => ({
                // get address from instruction
                let base_ram_address = 0xFF00;
                let immediate_address = self.pc.wrapping_add(1);
                let ram_offset = peripheral.read(immediate_address) as u16;

                if load {
                    // read data from ram memory & load it in register a
                    self.registers.a = peripheral.read(base_ram_address + ram_offset);
                } else {
                    // read data from register A & store it in RAM
                    peripheral
                        .write(base_ram_address + ram_offset, self.registers.a);
                }

                // return next program counter value
                self.pc.wrapping_add(2)
            }, RUN_3_CYCLES),
            RamTarget::AddressFromRegister => ({
                // get address from instruction
                let base_ram_address = 0xFF00;
                let ram_offset = self.registers.c as u16;

                if load {
                    // read data from ram memory & load it in register a
                    self.registers.a = peripheral.read(base_ram_address + ram_offset);
                } else {
                    // read data from register A & store it in RAM
                    peripheral
                        .write(base_ram_address + ram_offset, self.registers.a);
                }

                // return next program counter value
                self.pc.wrapping_add(1)
            }, RUN_2_CYCLES),
            RamTarget::TwoBytesAddress => ({
                // get address from instruction
                let low_byte_address = peripheral.read(self.pc.wrapping_add(1)) as u16;
                let high_byte_address = peripheral.read(self.pc.wrapping_add(2)) as u16;
                let address = low_byte_address + (high_byte_address << 8);

                if load {
                    // read data from ram memory & load it in register a
                    self.registers.a = peripheral.read(address);
                } else {
                    // read data from register A & store it in RAM
                    peripheral.write(address, self.registers.a);
                }

                // return next program counter value
                self.pc.wrapping_add(3)
            }, RUN_4_CYCLES),
        }
    }

    fn jump_relative(&mut self, flag: bool, peripheral: &mut Peripheral) -> (u16, u8) {
        // get the immediate from memory
        let immediate_address = self.pc.wrapping_add(1);
        let immediate = peripheral.read(immediate_address) as i8;

        // do the jump following the flag value
        if flag {
            // manage signed value to add to PC
            if immediate >= 0 {
                (self.pc.wrapping_add(2).wrapping_add(immediate as u16), RUN_3_CYCLES)
            } else {
                // using wrapping_sub() implies to convert immediate to absolute value
                (self.pc.wrapping_add(2).wrapping_sub(immediate.abs() as u16), RUN_3_CYCLES)
            }
        } else {
            (self.pc.wrapping_add(2), RUN_2_CYCLES)
        }
    }

    fn jump_immediate(&mut self, flag: bool, peripheral: &mut Peripheral) -> (u16, u8) {
        // get the immediate from memory
        let low_immediate = peripheral.read(self.pc.wrapping_add(1)) as u16;
        let high_immediate = peripheral.read(self.pc.wrapping_add(2)) as u16;
        let immediate = (high_immediate << 8) | low_immediate;

        // do the jump following the flag value
        if flag {
            (immediate, RUN_4_CYCLES)
        } else {
            (self.pc.wrapping_add(3), RUN_3_CYCLES)
        }
    }

    fn jump_indirect(&mut self) -> u16 {
        // get the immediate from memory
        self.registers.read_hl()
    }

    fn pop(&mut self, peripheral: &mut Peripheral) -> u16 {
        // get stack pointer values
        let low_stack_address = self.sp;
        let high_stack_address = self.sp.wrapping_add(1);
        // update stack pointer
        self.sp = self.sp.wrapping_add(2);
        // read data from RAM memory
        let low_byte = peripheral.read(low_stack_address) as u16;
        let high_byte = peripheral.read(high_stack_address) as u16;
        low_byte | (high_byte << 8)
    }

    fn push(&mut self, push_data: u16, peripheral: &mut Peripheral) {
        // get bytes from data
        let high_byte = ((push_data & 0xFF00) >> 8) as u8;
        let low_byte = (push_data & 0x00FF) as u8;
        // get stack pointer values
        let high_stack_address = self.sp.wrapping_sub(1);
        let low_stack_address = self.sp.wrapping_sub(2);
        // save data in memory
        peripheral.write(high_stack_address, high_byte);
        peripheral.write(low_stack_address, low_byte);
        // update stack pointer
        self.sp = self.sp.wrapping_sub(2);
    }

    fn add_sp(&mut self, peripheral: &mut Peripheral) -> u16 {
        let address = self.pc.wrapping_add(1);
        let value = peripheral.read(address) as i8 as i16 as u16;
        self.sp = self.sp.wrapping_add(value);

        // update flags
        self.registers.f.zero = false;
        self.registers.f.substraction = false;
        self.registers.f.half_carry = (self.sp & 0xF) + (value & 0xF) > 0xF;
        self.registers.f.carry = (self.sp & 0xFF) + (value & 0xFF) > 0xFF;

        // return next program counter value
        self.pc.wrapping_add(2)
    }

    fn reset(&mut self, addr_to_reset: u8, peripheral: &mut Peripheral) -> u16 {
        // save PC value on the stack
        self.push(self.pc.wrapping_add(1), peripheral);
        // return next PC value
        addr_to_reset as u16
    }

    fn reti(&mut self, peripheral: &mut Peripheral) -> u16 {
        peripheral.nvic.master_enable(true);
        self.pop(peripheral)
    }

    fn call(&mut self, flag: bool, peripheral: &mut Peripheral) -> (u16, u8) {
        // do the call following the flag value
        if flag {
            // save the return address on the stack
            self.push(self.pc.wrapping_add(3), peripheral);
            // get the call address
            let low_byte_address = peripheral.read(self.pc.wrapping_add(1)) as u16;
            let high_byte_address = peripheral.read(self.pc.wrapping_add(2)) as u16;
            let call_address = low_byte_address | (high_byte_address << 8);
            // return the call address
            (call_address, RUN_6_CYCLES)
        } else {
            // go to the next instruction
            (self.pc.wrapping_add(3), RUN_3_CYCLES)
        }
    }

    fn set_cpu_mode(&mut self, mode: CpuMode) -> u16 {
        self.mode = mode;
        self.pc.wrapping_add(1)
    }

    fn flip_register_a(&mut self) -> u16 {
        // flip register a
        self.registers.a = !self.registers.a;

        // update flag register
        self.registers.f.substraction = true;
        self.registers.f.half_carry = true;

        // return next pc value
        self.pc.wrapping_add(1)
    }

    fn set_carry(&mut self, operation: CarryOp) -> u16 {
        // set carry depending on operation value
        match operation {
            CarryOp::SET => self.registers.f.carry = true,
            CarryOp::FLIP => self.registers.f.carry = !self.registers.f.carry,
        }

        // update flags
        self.registers.f.zero = false;
        self.registers.f.half_carry = false;

        // return next pc value
        self.pc.wrapping_add(1)
    }

    fn decimal_adjust(&mut self) -> u16 {
        // huge help from https://github.com/rylev/DMG-01/blob/master/lib-dmg-01/src/cpu/mod.rs

        let flags = self.registers.f;
        let mut carry = false;

        // adjust
        let result = if !flags.substraction {
            let mut result = self.registers.a;
            if flags.carry || self.registers.a > 0x99 {
                carry = true;
                result = result.wrapping_add(0x60);
            }
            if flags.half_carry || self.registers.a & 0x0F > 0x09 {
                result = result.wrapping_add(0x06);
            }
            result
        } else if flags.carry {
            carry = true;
            let add = if flags.half_carry { 0x9A } else { 0xA0 };
            self.registers.a.wrapping_add(add)
        } else if flags.half_carry {
            self.registers.a.wrapping_add(0xFA)
        } else {
            self.registers.a
        };
        // update a register with the new value
        self.registers.a = result;

        // update flags
        self.registers.f.zero = result == 0;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = false;

        // return next pc value
        self.pc.wrapping_add(1)
    }

    fn rotate(&mut self, value: u8, direction: Direction, zero: bool) -> u8 {
        match direction {
            Direction::LEFT => {
                // save bit 7
                let last_bit = (value & 0b1000_0000) >> 7;
                // shift register
                let output_value = (value << 1) | last_bit;
                // update zero flag
                if zero {
                    self.registers.f.zero = output_value == 0;
                } else {
                    self.registers.f.zero = false;
                }
                // update carry
                self.registers.f.carry = last_bit != 0;
                // return computed value
                output_value
            }
            Direction::RIGHT => {
                // save bit 0
                let first_bit = (value & 0b0000_0001) << 7;
                // shift register
                let output_value = (value >> 1) | first_bit;
                // update zero flag
                if zero {
                    self.registers.f.zero = output_value == 0;
                } else {
                    self.registers.f.zero = false;
                }
                // update carry
                self.registers.f.carry = first_bit != 0;
                // return computed value
                output_value
            }
        }
    }

    fn rotate_through_carry(&mut self, value: u8, direction: Direction, zero: bool) -> u8 {
        match direction {
            Direction::LEFT => {
                // save bit 7
                let last_bit = (value & 0b1000_0000) >> 7;
                // shift register
                let output_value = (value << 1) | (self.registers.f.carry as u8);
                // update zero flag
                if zero {
                    self.registers.f.zero = output_value == 0;
                } else {
                    self.registers.f.zero = false;
                }
                // update carry
                self.registers.f.carry = last_bit != 0;
                // return computed value
                output_value
            }
            Direction::RIGHT => {
                // save bit 0
                let first_bit = (value & 0b0000_0001) << 7;
                // shift register
                let output_value = (value >> 1) | ((self.registers.f.carry as u8) << 7);
                // update zero flag
                if zero {
                    self.registers.f.zero = output_value == 0;
                } else {
                    self.registers.f.zero = false;
                }
                // update carry
                self.registers.f.carry = first_bit != 0;
                // return computed value
                output_value
            }
        }
    }

    fn shift_left_and_reset(&mut self, value: u8) -> u8 {
        // save bit 7
        let last_bit = (value & 0b1000_0000) >> 7;
        // shift register
        let output_value = value << 1;
        // update flag register
        self.registers.f.substraction = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = output_value == 0;
        // update carry
        self.registers.f.carry = last_bit != 0;
        // return computed value
        output_value
    }

    fn shift_right_and_reset(&mut self, value: u8) -> u8 {
        // save bit 0
        let first_bit = (value & 0b0000_0001) << 7;
        // shift register
        let output_value = value >> 1;
        // update flag register
        self.registers.f.substraction = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = output_value == 0;
        // update carry
        self.registers.f.carry = first_bit != 0;
        // return computed value
        output_value
    }

    fn shift_right(&mut self, value: u8) -> u8 {
        // save bit 0
        let first_bit = (value & 0b0000_0001) << 7;
        // shift register
        let output_value = (value >> 1) | (value & 0b1000_0000);
        // update flag register
        self.registers.f.substraction = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = output_value == 0;
        // update carry
        self.registers.f.carry = first_bit != 0;
        // return computed value
        output_value
    }

    fn swap(&mut self, value: u8) -> u8 {
        // save bit 0
        let low_bits = value & 0x0F;
        let high_bits = value & 0xF0;
        // shift register
        let output_value = (low_bits << 4) | (high_bits >> 4);
        // update flag register
        self.registers.f.substraction = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = output_value == 0;
        self.registers.f.carry = false;
        // return computed value
        output_value
    }

    fn complement_bit(&mut self, bit: u8, value: u8) -> u16 {
        // get bit value
        let bit_value = (value >> bit) & 0x01;
        // update flag register
        self.registers.f.substraction = false;
        self.registers.f.half_carry = true;
        self.registers.f.zero = bit_value == 0;
        // return next pc
        self.pc.wrapping_add(2)
    }

    fn set_bit(&mut self, enable: bool, bit: u8, value: u8) -> u8 {
        // set / reset bit
        if enable {
            value | ((0x01 as u8) << bit)
        } else {
            value & !((0x01 as u8) << bit)
        }
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::*;
    use crate::soc::cpu::instruction::ArithmeticTarget::{B, C, D, D8, E, H, HL};
    use crate::soc::cpu::instruction::Instruction::{
        ADD, ADD16, ADDC, AND, CP, DEC, DEC16, DI, EI, INC, INC16, LOAD, LOAD_IMMEDIATE,
        LOAD_INDIRECT, LOAD_SP, OR, POP, PUSH, RESET, RETI, RETURN, SBC, STORE_INDIRECT, SUB, XOR,
    };
    use crate::soc::cpu::instruction::{
        IncDecTarget, JumpTarget, Load16Target, PopPushTarget, ResetTarget, SPTarget, U16Target,
    };

    #[test]
    fn test_add_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0xAABB);
        cpu.execute(ADD(B), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0xAA00);
    }

    #[test]
    fn test_add_memory() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        let address = 0x1234;
        let data = 0xAA;

        peripheral.write(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(ADD(HL), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0xAA00);
    }

    #[test]
    fn test_add_immediate() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        let address = 0x0001;
        let data = 0x23;

        peripheral.write(address, data);
        cpu.execute(ADD(D8), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x2300);
    }

    #[test]
    fn test_add16_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0x2200);
        cpu.registers.write_hl(0x0125);
        cpu.execute(ADD16(U16Target::BC), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), 0x2325);

        cpu.registers.write_de(0x00FF);
        cpu.registers.write_hl(0xFF01);
        cpu.execute(ADD16(U16Target::DE), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), 0x0000);

        cpu.registers.write_hl(0xF025);
        cpu.execute(ADD16(U16Target::HL), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), 0xE04A);

        cpu.sp = 0x0001;
        cpu.registers.write_hl(0xF025);
        cpu.execute(ADD16(U16Target::SP), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), 0xF026);
    }

    #[test]
    fn test_addc_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_af(0x0110);
        cpu.registers.write_bc(0xAABB);
        cpu.execute(ADDC(B), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0xAC00);

        cpu.registers.write_af(0x0110);
        cpu.registers.write_bc(0xFF00);
        cpu.execute(ADDC(B), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x0130);
    }

    #[test]
    fn test_addc_memory() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        let address = 0x1234;
        let data = 0xAA;

        peripheral.write(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(ADDC(HL), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0xAA00);
    }

    #[test]
    fn test_addc_immediate() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        let address = 0x0001;
        let data = 0x23;

        peripheral.write(address, data);
        cpu.registers.write_af(0x0110);
        cpu.execute(ADDC(D8), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x2500);
    }

    #[test]
    fn test_sub_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0xAABB);
        cpu.registers.write_af(0xFF00);
        cpu.execute(SUB(C), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x4440);
    }

    #[test]
    fn test_subc_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0xAABB);
        cpu.registers.write_af(0xFF10);
        cpu.execute(SBC(C), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x4540);
    }

    #[test]
    fn test_and_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0xAABB);
        cpu.registers.write_af(0xAA00);
        cpu.execute(AND(B), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0xAA20);
    }

    #[test]
    fn test_xor_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0x0022);
        cpu.registers.write_af(0x2100);
        cpu.execute(XOR(C), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x0300);
    }

    #[test]
    fn test_or_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();
        cpu.registers.write_bc(0x0022);
        cpu.registers.write_af(0x2100);
        cpu.execute(OR(C), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x2300);
    }

    #[test]
    fn test_cp_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_bc(0x0022);
        cpu.registers.write_af(0x2200);
        cpu.execute(CP(C), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x22C0);

        cpu.registers.write_bc(0x0033);
        cpu.registers.write_af(0x2200);
        cpu.execute(CP(C), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x2270);
    }

    #[test]
    fn test_inc_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_bc(0x2200);
        cpu.execute(INC(IncDecTarget::B), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x2300);

        cpu.registers.write_bc(0x22FF);
        cpu.execute(INC(IncDecTarget::C), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x2200);

        let address = 0x1234;
        let data = 0xAA;
        peripheral.write(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(INC(IncDecTarget::HL), &mut peripheral);
        assert_eq!(peripheral.read(address), 0xAB);
    }

    #[test]
    fn test_inc16_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_bc(0x2200);
        cpu.execute(INC16(U16Target::BC), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x2201);

        cpu.registers.write_de(0x22FF);
        cpu.execute(INC16(U16Target::DE), &mut peripheral);
        assert_eq!(cpu.registers.read_de(), 0x2300);

        cpu.registers.write_hl(0xFFFF);
        cpu.execute(INC16(U16Target::HL), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), 0x0000);

        cpu.sp = 0x3578;
        cpu.execute(INC16(U16Target::SP), &mut peripheral);
        assert_eq!(cpu.sp, 0x3579);
    }

    #[test]
    fn test_dec_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_bc(0x2200);
        cpu.execute(DEC(IncDecTarget::B), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x2100);

        cpu.registers.write_bc(0x2200);
        cpu.execute(DEC(IncDecTarget::C), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x22FF);

        let address = 0x1234;
        let data = 0xAA;
        peripheral.write(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(DEC(IncDecTarget::HL), &mut peripheral);
        assert_eq!(peripheral.read(address), 0xA9);
    }

    #[test]
    fn test_dec16_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_bc(0x2200);
        cpu.execute(DEC16(U16Target::BC), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x21FF);

        cpu.registers.write_de(0x0000);
        cpu.execute(DEC16(U16Target::DE), &mut peripheral);
        assert_eq!(cpu.registers.read_de(), 0xFFFF);

        cpu.registers.write_hl(0x1279);
        cpu.execute(DEC16(U16Target::HL), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), 0x1278);

        cpu.sp = 0x0001;
        cpu.execute(DEC16(U16Target::SP), &mut peripheral);
        assert_eq!(cpu.sp, 0x0000);
    }

    #[test]
    fn test_load_registers() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.write_de(0x0057);
        cpu.execute(LOAD(IncDecTarget::B, E), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x5700);

        cpu.registers.write_hl(0x6400);
        cpu.execute(LOAD(IncDecTarget::C, H), &mut peripheral);
        assert_eq!(cpu.registers.read_bc(), 0x5764);

        let mut mem_address = 0x0001;
        let mut data = 0x23;
        peripheral.write(mem_address, data);
        cpu.execute(LOAD(IncDecTarget::A, D8), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x2300);

        mem_address = 0x0010;
        cpu.registers.write_hl(mem_address);
        cpu.execute(LOAD(IncDecTarget::HL, D8), &mut peripheral);
        assert_eq!(peripheral.read(mem_address), 0x23);

        mem_address = 0x002A;
        cpu.registers.write_hl(mem_address);
        cpu.registers.write_de(0xD500);
        cpu.execute(LOAD(IncDecTarget::HL, D), &mut peripheral);
        assert_eq!(peripheral.read(mem_address), 0xD5);

        mem_address = 0x00C8;
        data = 0x5F;
        peripheral.write(mem_address, data);
        cpu.registers.write_hl(mem_address);
        cpu.execute(LOAD(IncDecTarget::A, HL), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x5F00);

        cpu.registers.write_de(0x0000);
        cpu.execute(LOAD(IncDecTarget::E, HL), &mut peripheral);
        assert_eq!(cpu.registers.read_de(), 0x005F);
    }

    #[test]
    fn test_load_indirect() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let mem_address = 0x00D8;
        let mut data = 0x56;
        peripheral.write(mem_address, data);
        cpu.registers.write_bc(mem_address);
        cpu.execute(LOAD_INDIRECT(Load16Target::BC), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0x5600);

        data = 0xC6;
        peripheral.write(mem_address, data);
        cpu.registers.write_hl(mem_address);
        cpu.execute(LOAD_INDIRECT(Load16Target::HL_plus), &mut peripheral);
        assert_eq!(cpu.registers.read_af(), 0xC600);
        assert_eq!(cpu.registers.read_hl(), 0x00D9);
    }

    #[test]
    fn test_load_immediate() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let low_data = 0x4C;
        let high_data = 0xB7;
        let value = ((high_data as u16) << 8) + low_data as u16;
        peripheral.write(0x0001, low_data);
        peripheral.write(0x0002, high_data);
        cpu.execute(LOAD_IMMEDIATE(U16Target::DE), &mut peripheral);
        assert_eq!(cpu.registers.read_de(), value);

        cpu.execute(LOAD_IMMEDIATE(U16Target::SP), &mut peripheral);
        assert_eq!(cpu.sp, value);
    }

    #[test]
    fn test_store_indirect() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let mem_address = 0x00D8;
        let mut data = 0x5600;
        cpu.registers.write_af(data);
        cpu.registers.write_de(mem_address);
        cpu.execute(STORE_INDIRECT(Load16Target::DE), &mut peripheral);
        assert_eq!(peripheral.read(mem_address), 0x56);

        data = 0xC600;
        cpu.registers.write_af(data);
        cpu.registers.write_hl(mem_address);
        cpu.execute(STORE_INDIRECT(Load16Target::HL_minus), &mut peripheral);
        assert_eq!(peripheral.read(mem_address), 0xC6);
        assert_eq!(cpu.registers.read_hl(), 0x00D7);
    }

    #[test]
    fn test_jump_relative_nzero() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let base_address: u16 = 0x0000;
        let jump: u8 = 0x05;
        let jump_nz: u8 = 0x20;
        let program: [u8; 10] = [
            jump_nz, jump, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(base_address + index, data);
            index += 1;
        }

        // run CPU to do the jump
        cpu.run(&mut peripheral);
        assert_eq!(
            peripheral.read(cpu.pc),
            peripheral.read(base_address + (jump as u16) + 2)
        );

        // reset CPU and run it with the flag, we don't do the jump
        cpu.registers.f.zero = true;
        cpu.pc = base_address;
        cpu.run(&mut peripheral);
        assert_eq!(peripheral.read(cpu.pc), peripheral.read(base_address + 2));
    }

    #[test]
    fn test_jump_relative_carry() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let base_address: u16 = 0x0000;
        let jump: u8 = 0x05;
        let jump_carry: u8 = 0x38;
        let program: [u8; 10] = [
            jump_carry, jump, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(base_address + index, data);
            index += 1;
        }

        // run CPU to do the jump
        cpu.registers.f.carry = true;
        cpu.run(&mut peripheral);
        assert_eq!(
            peripheral.read(cpu.pc),
            peripheral.read(base_address + (jump as u16) + 2)
        );

        // reset CPU and run it with the flag, we don't do the jump
        cpu.registers.f.carry = false;
        cpu.pc = base_address;
        cpu.run(&mut peripheral);
        assert_eq!(peripheral.read(cpu.pc), peripheral.read(base_address + 2));
    }

    #[test]
    fn test_jump_relative_immediate() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let base_address: u16 = 0x0000;
        let jump: u8 = 0x06;
        let jump_inst: u8 = 0x18;
        let program: [u8; 10] = [
            jump_inst, jump, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(base_address + index, data);
            index += 1;
        }

        // run CPU to do the jump
        cpu.run(&mut peripheral);
        assert_eq!(
            peripheral.read(cpu.pc),
            peripheral.read(base_address + (jump as u16) + 2)
        );
    }

    #[test]
    fn test_jump_immediate_zero() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let base_address: u16 = 0x0000;
        let jump: u8 = 0x05;
        let jump_carry: u8 = 0xCA;
        let program: [u8; 10] = [
            jump_carry, jump, 0x00, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(base_address + index, data);
            index += 1;
        }

        // run CPU to do the jump
        cpu.registers.f.zero = true;
        cpu.run(&mut peripheral);
        assert_eq!(
            peripheral.read(cpu.pc),
            peripheral.read(base_address + (jump as u16))
        );

        // reset CPU and run it with the flag, we don't do the jump
        cpu.registers.f.zero = false;
        cpu.pc = base_address;
        cpu.run(&mut peripheral);
        assert_eq!(peripheral.read(cpu.pc), peripheral.read(base_address + 3));
    }

    #[test]
    fn test_jump_indirect() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let jump_inst: u8 = 0xE9;
        let program: [u8; 8] = [jump_inst, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let mut index = 0;
        for data in program {
            peripheral.write(index, data);
            index += 1;
        }

        let jump: u16 = 0x05;
        cpu.registers.write_hl(jump);
        // run CPU to do the jump
        cpu.run(&mut peripheral);
        assert_eq!(peripheral.read(cpu.pc), peripheral.read(jump));
    }

    #[test]
    fn test_load_from_hl_to_sp() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let data: u16 = 0xA7D8;
        cpu.registers.write_hl(data);
        cpu.execute(LOAD_SP(SPTarget::TO_SP), &mut peripheral);
        assert_eq!(cpu.pc, data);
    }

    #[test]
    fn test_load_from_sp_to_hl() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.sp = 0x0010;
        let offset: u8 = 0x02;

        // first, fill memory with program
        let jump_inst: u8 = 0xF8;
        let program: [u8; 2] = [jump_inst, offset];
        let mut index = 0;
        for data in program {
            peripheral.write(index, data);
            index += 1;
        }

        cpu.run(&mut peripheral);
        assert_eq!(cpu.registers.read_hl(), cpu.sp + offset as u16);
    }

    #[test]
    fn test_load_from_sp_to_mem() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let base_address = 0x24C8;
        let jump_inst: u8 = 0x08;
        let low_address = 0x05;
        let high_address = 0xA1;
        let address = (low_address as u16) + ((high_address as u16) << 8);
        let program: [u8; 3] = [jump_inst, low_address, high_address];
        let mut index = 0;
        for data in program {
            peripheral.write(index + base_address, data);
            index += 1;
        }

        // set cpu and run it
        cpu.pc = base_address;
        cpu.sp = 0x57A8;
        cpu.run(&mut peripheral);
        assert_eq!((cpu.sp & 0x00FF) as u8, peripheral.read(address));
        assert_eq!(
            ((cpu.sp & 0xFF00) >> 8) as u8,
            peripheral.read(address + 1)
        );
    }

    #[test]
    fn test_load_ram_from_one_byte() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // initialize RAM memory
        let ram_data_address = 0xFFA5;
        let data = 0xF8;
        peripheral.write(ram_data_address, data);

        // initialize ROM memory
        let base_program_address = 0x0000;
        let jump_inst: u8 = 0xF0;
        let program: [u8; 2] = [jump_inst, (ram_data_address & 0x00FF) as u8];
        let mut index = 0;
        for data in program {
            peripheral.write(index + base_program_address, data);
            index += 1;
        }

        // set cpu and run it
        cpu.pc = base_program_address;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.registers.a, peripheral.read(ram_data_address));
    }

    #[test]
    fn test_load_ram_from_two_bytes() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // initialize RAM memory
        let ram_data_address = 0xFFA5;
        let data = 0xF8;
        peripheral.write(ram_data_address, data);

        // initialize ROM memory
        let base_program_address = 0x0000;
        let jump_inst: u8 = 0xFA;
        let program: [u8; 3] = [
            jump_inst,
            (ram_data_address & 0x00FF) as u8,
            ((ram_data_address & 0xFF00) >> 8) as u8,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(index + base_program_address, data);
            index += 1;
        }

        // set cpu and run it
        cpu.pc = base_program_address;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.registers.a, peripheral.read(ram_data_address));
    }

    #[test]
    fn test_load_ram_from_register() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // initialize RAM memory
        let ram_data_address = 0xFFA5;
        let data = 0xF8;
        peripheral.write(ram_data_address, data);

        // initialize ROM memory
        let base_program_address = 0x0000;
        let jump_inst: u8 = 0xF2;
        let program: [u8; 1] = [jump_inst];
        let mut index = 0;
        for data in program {
            peripheral.write(index + base_program_address, data);
            index += 1;
        }

        // set cpu and run it
        cpu.pc = base_program_address;
        cpu.registers.c = (ram_data_address & 0x00FF) as u8;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.registers.a, peripheral.read(ram_data_address));
    }

    #[test]
    fn test_store_ram_from_one_byte() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // initialize RAM memory
        let ram_data_address = 0xFFA5;
        let data = 0xF8;

        // initialize ROM memory
        let base_program_address = 0x0000;
        let jump_inst: u8 = 0xE0;
        let program: [u8; 2] = [jump_inst, (ram_data_address & 0x00FF) as u8];
        let mut index = 0;
        for data in program {
            peripheral.write(index + base_program_address, data);
            index += 1;
        }

        // set cpu and run it
        cpu.pc = base_program_address;
        cpu.registers.a = data;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.registers.a, peripheral.read(ram_data_address));
    }

    #[test]
    fn test_push_and_pop() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // initialize RAM memory parameters
        let ram_address = 0xFFA5;
        let push_data = 0xD7F8;

        // test push instruction
        cpu.sp = ram_address;
        cpu.registers.write_de(push_data);
        cpu.execute(PUSH(PopPushTarget::DE), &mut peripheral);
        assert_eq!(
            peripheral.read(ram_address.wrapping_sub(1)),
            ((push_data & 0xFF00) >> 8) as u8
        );
        assert_eq!(
            peripheral.read(ram_address.wrapping_sub(2)),
            (push_data & 0x00FF) as u8
        );

        // test pop instruction
        cpu.execute(POP(PopPushTarget::HL), &mut peripheral);
        assert_eq!(cpu.registers.read_hl(), push_data);
    }

    #[test]
    fn test_add_sp() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // init parameters
        let data_to_add = 0x88;
        let sp_init = 0xF147;

        // initialize ROM memory
        let base_program_address = 0x0000;
        let inst: u8 = 0xE8;
        let program: [u8; 2] = [inst, data_to_add as u8];
        let mut index = 0;
        for data in program {
            peripheral.write(index + base_program_address, data);
            index += 1;
        }

        // set cpu and run it
        cpu.pc = base_program_address;
        cpu.sp = sp_init;
        cpu.run(&mut peripheral);
        assert_eq!(
            sp_init.wrapping_add(data_to_add as i8 as i16 as u16),
            cpu.sp
        );
    }

    #[test]
    fn test_return() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // initialize RAM memory parameters
        let ram_address = 0xFFA5;
        let push_data = 0xD7F8;

        // test push instruction
        cpu.sp = ram_address;
        cpu.registers.write_de(push_data);
        cpu.execute(PUSH(PopPushTarget::DE), &mut peripheral);
        let (next_pc, cycles) = cpu.execute(RETURN(JumpTarget::IMMEDIATE), &mut peripheral);
        assert_eq!(next_pc, push_data);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // test push instruction
        cpu.sp = 0xFFAF;
        let (next_pc, cycles) = cpu.execute(RESET(ResetTarget::FLASH_1), &mut peripheral);
        assert_eq!(next_pc, 0x08);
    }

    #[test]
    fn test_interrupt() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.execute(EI, &mut peripheral);
        assert_eq!(peripheral.nvic.interrupt_master_enable, true);

        cpu.execute(DI, &mut peripheral);
        assert_eq!(peripheral.nvic.interrupt_master_enable, false);

        // initialize RAM memory parameters
        let ram_address = 0xFFA5;
        let push_data = 0xD7F8;

        // test push instruction
        cpu.sp = ram_address;
        cpu.registers.write_de(push_data);
        cpu.execute(PUSH(PopPushTarget::DE), &mut peripheral);
        let (next_pc, cycles) = cpu.execute(RETI, &mut peripheral);
        assert_eq!(next_pc, push_data);
        assert_eq!(peripheral.nvic.interrupt_master_enable, true);
    }

    #[test]
    fn test_call() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let inst: u8 = 0xC4;
        let program: [u8; 8] = [inst, 0x00, 0x05, 0x44, 0x55, 0x66, 0x77, 0x88];
        let mut index = 0;
        for data in program {
            peripheral.write(index, data);
            index += 1;
        }

        // run CPU to do the jump
        let ram_address = 0xFFA5;
        cpu.sp = ram_address;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0500);
    }

    #[test]
    fn test_nop_stop_halt() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // first, fill memory with program
        let nop_inst: u8 = 0x00;
        let stop_inst: u8 = 0x10;
        let halt_inst: u8 = 0x76;
        let program: [u8; 8] = [
            nop_inst, stop_inst, nop_inst, halt_inst, nop_inst, nop_inst, nop_inst, nop_inst,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(index, data);
            index += 1;
        }

        // run CPU to do the NOP
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0001);
        // run CPU to do the STOP
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0002);
        // then CPU is blocked
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0002);

        // Unlock CPU and run NOP inst
        cpu.mode = CpuMode::RUN;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0003);
        // run HALT inst
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0004);
        // cpu is blocked
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0004);
    }

    #[test]
    fn test_jump_to_interrupt() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        // init stack pointer
        cpu.sp = 0xFFA5;

        // first, fill memory with program
        let nop_inst: u8 = 0x00;
        let stop_inst: u8 = 0x10;
        let halt_inst: u8 = 0x76;
        let program: [u8; 8] = [
            nop_inst, stop_inst, nop_inst, halt_inst, nop_inst, nop_inst, nop_inst, nop_inst,
        ];
        let mut index = 0;
        for data in program {
            peripheral.write(index, data);
            index += 1;
        }

        // run CPU to do the NOP
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0001);
        // run CPU to do the STOP
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0002);
        // then CPU is blocked
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0002);

        // Unlock CPU and run NOP inst
        cpu.mode = CpuMode::RUN;
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0003);
        // run HALT inst
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0004);
        // cpu is blocked
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, 0x0004);

        peripheral.nvic.master_enable(true);
        peripheral.nvic.enable_interrupt(InterruptSources::STAT, true);
        peripheral.nvic.set_interrupt(InterruptSources::STAT);
        cpu.run(&mut peripheral);
        assert_eq!(cpu.pc, LCDSTAT_VECTOR);
    }

    #[test]
    fn test_complement() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.a = 0x55;
        cpu.execute(Instruction::CPL, &mut peripheral);
        assert_eq!(cpu.registers.a, 0xAA);
    }

    #[test]
    fn test_set_carry() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.execute(Instruction::SCF, &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        cpu.execute(Instruction::CCF, &mut peripheral);
        assert_eq!(cpu.registers.f.carry, false);
        cpu.execute(Instruction::CCF, &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
    }

    #[test]
    fn test_decimal_adjust() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.a = 0x0B;
        cpu.execute(Instruction::DAA, &mut peripheral);
        assert_eq!(cpu.registers.a, 0x11);
    }

    #[test]
    fn test_rotate_left() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.a = 0xB5;
        cpu.execute(Instruction::RCA(Direction::LEFT), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.a, 0x6B);
    }

    #[test]
    fn test_rotate_through_carry_right() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.a = 0xB5;
        cpu.registers.f.carry = true;
        cpu.execute(Instruction::RA(Direction::RIGHT), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.a, 0xDA);
    }

    #[test]
    fn test_decode_long_instruction() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let program: [u8; 2] = [0xCB, 0x19];
        let mut index = 0;
        for data in program {
            peripheral.write(index, data);
            index += 1;
        }

        if let Some(instruction) = cpu.decode(0xCB, &mut peripheral) {
            assert_eq!(
                instruction,
                Instruction::R(Direction::RIGHT, IncDecTarget::C)
            );
        } else {
            panic!("Unkown long instruction");
        }
    }

    #[test]
    fn test_long_rotate() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.b = 0xB5;
        cpu.execute(Instruction::RC(Direction::LEFT, IncDecTarget::B), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.b, 0x6B);
    }

    #[test]
    fn test_long_rotate_hl() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let address = 0x1234;
        let data = 0xB5;
        peripheral.write(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(Instruction::RC(Direction::LEFT, IncDecTarget::HL), &mut peripheral);
        assert_eq!(peripheral.read(address), 0x6B);
    }

    #[test]
    fn test_long_rotate_through_carry_right() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.e = 0xB5;
        cpu.registers.f.carry = true;
        cpu.execute(Instruction::R(Direction::RIGHT, IncDecTarget::E), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.e, 0xDA);
    }

    #[test]
    fn test_shift_left_and_reset() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.d = 0xB5;
        cpu.execute(Instruction::SLA(IncDecTarget::D), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.d, 0x6A);
    }

    #[test]
    fn test_shift_right_and_reset() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.h = 0xB5;
        cpu.execute(Instruction::SRL(IncDecTarget::H), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.h, 0x5A);
    }

    #[test]
    fn test_shift_right_and_reset_hl() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let address = 0x1234;
        let data = 0xB5;
        peripheral.write(address, data);
        cpu.registers.write_hl(address);
        cpu.execute(Instruction::SRL(IncDecTarget::HL), &mut peripheral);
        assert_eq!(peripheral.read(address), 0x5A);
    }

    #[test]
    fn test_shift_right() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.c = 0xB5;
        cpu.execute(Instruction::SRA(IncDecTarget::C), &mut peripheral);
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.c, 0xDA);
    }

    #[test]
    fn test_swap() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.l = 0xB5;
        cpu.execute(Instruction::SWAP(IncDecTarget::L), &mut peripheral);
        assert_eq!(cpu.registers.l, 0x5B);
    }

    #[test]
    fn test_complement_bit() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.h = 0xB5;
        cpu.execute(Instruction::BIT(BitTarget::BIT_1, IncDecTarget::H), &mut peripheral);
        assert_eq!(cpu.registers.f.zero, true);
        cpu.execute(Instruction::BIT(BitTarget::BIT_4, IncDecTarget::H), &mut peripheral);
        assert_eq!(cpu.registers.f.zero, false);
        cpu.execute(Instruction::BIT(BitTarget::BIT_6, IncDecTarget::H), &mut peripheral);
        assert_eq!(cpu.registers.f.zero, true);

        cpu.registers.d = 0x5B;
        cpu.execute(Instruction::BIT(BitTarget::BIT_1, IncDecTarget::D), &mut peripheral);
        assert_eq!(cpu.registers.f.zero, false);
        cpu.execute(Instruction::BIT(BitTarget::BIT_4, IncDecTarget::D), &mut peripheral);
        assert_eq!(cpu.registers.f.zero, false);
        cpu.execute(Instruction::BIT(BitTarget::BIT_5, IncDecTarget::D), &mut peripheral);
        assert_eq!(cpu.registers.f.zero, true);
    }

    #[test]
    fn test_set_reset_bit() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        cpu.registers.b = 0xB5;
        cpu.execute(Instruction::RESET_BIT(BitTarget::BIT_2, IncDecTarget::B), &mut peripheral);
        assert_eq!(cpu.registers.b, 0xB1);
        cpu.execute(Instruction::SET_BIT(BitTarget::BIT_3, IncDecTarget::B), &mut peripheral);
        assert_eq!(cpu.registers.b, 0xB9);
        cpu.execute(Instruction::RESET_BIT(BitTarget::BIT_5, IncDecTarget::B), &mut peripheral);
        assert_eq!(cpu.registers.b, 0x99);
    }

    #[test]
    fn test_set_reset_bit_hl() {
        let mut cpu = Cpu::new();
        let mut peripheral = Peripheral::new();

        let address = 0x1234;
        let data = 0xB5;
        peripheral.write(address, data);
        cpu.registers.write_hl(address);

        cpu.execute(Instruction::RESET_BIT(BitTarget::BIT_2, IncDecTarget::HL), &mut peripheral);
        assert_eq!(peripheral.read(address), 0xB1);

        cpu.execute(Instruction::SET_BIT(BitTarget::BIT_3, IncDecTarget::HL), &mut peripheral);
        assert_eq!(peripheral.read(address), 0xB9);
    }
}