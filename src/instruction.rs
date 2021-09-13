mod register;

struct Cpu {
    registers: Registers,
    pc: u16,
    bus: MemoryBus,
}

struct MemoryBus {
    memory: [u8, 0xFFFF]
}

impl MemoryBus {
    fn read_byte (&self, address: u16) -> u8 {
        self.memory[address]
    }
}

enum Instruction {
    ADD(ArithmeticTarget),
}

impl Instruction {
    fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            _ => None
        }
    }
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Cpu {

    fn step(&mut self) {
        let instruction_byte = self.bus.read_byte(self.pc);

        let next_pc = if let Some(instruction) = instruction::from_byte(instruction_byte) {
            self.execute(instruction)
        } else {
            panic!("Unknown instruction found for 0x{:x}", instruction_byte)
        }
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => {
                match target => {
                    ArithmeticTarget::C => {
                        let value = self.registers.c;
                        let new_value = self.add(value);
                        self.registers.a = new_value;

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