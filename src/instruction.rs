enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

enum Instruction {
    ADD(ArithmeticTarget),
    ADDC(ArithmeticTarget),
}

impl Instruction {
    fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            // ADD
            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
            0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
            0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
            0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
            0x86 => Some(Instruction::ADD(ArithmeticTarget::HL)),
            0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),

            // ADDC
            0x88 => Some(Instruction::ADDC(ArithmeticTarget::B)),
            0x89 => Some(Instruction::ADDC(ArithmeticTarget::C)),
            0x8A => Some(Instruction::ADDC(ArithmeticTarget::D)),
            0x8B => Some(Instruction::ADDC(ArithmeticTarget::E)),
            0x8C => Some(Instruction::ADDC(ArithmeticTarget::H)),
            0x8D => Some(Instruction::ADDC(ArithmeticTarget::L)),
            0x8E => Some(Instruction::ADDC(ArithmeticTarget::HL)),
            0x8F => Some(Instruction::ADDC(ArithmeticTarget::A)),

            _ => None,
        }
    }
}
