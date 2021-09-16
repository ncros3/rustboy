mod register;
mod instruction;
mod bus;

struct Cpu {
    registers: Registers,
    pc: u16,
    bus: MemoryBus,
}

impl Cpu {

    fn run(&mut self) {
        // fetch instruction
        let instruction_byte = self.bus.read_byte(self.pc);
        // decode instruction
        let next_pc = if let Some(instruction) = instruction::from_byte(instruction_byte) {
            // execute instruction
            self.execute(instruction)
        } else {
            panic!("Unknown instruction found for 0x{:x}", instruction_byte)
        }
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => {
                match target => {
                    ArithmeticTarget::A => {
                        let value = self.registers.a;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    ArithmeticTarget::B => {
                        let value = self.registers.b;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    ArithmeticTarget::C => {
                        let value = self.registers.c;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    ArithmeticTarget::D => {
                        let value = self.registers.d;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    ArithmeticTarget::E => {
                        let value = self.registers.e;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    ArithmeticTarget::H => {
                        let value = self.registers.h;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    ArithmeticTarget::L => {
                        let value = self.registers.l;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        // modulo operation to avoid overflowing effects
                        self.pc.wrapping_add(1)
                    }
                    _ => {
                        // TODO: support more targets
                        self.pc
                    }
                }
            }
            _ => {
                // TODO: support more instructions
                self.pc
            }
        }
    }

    fn add(&mut self, value: u8) {
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
}