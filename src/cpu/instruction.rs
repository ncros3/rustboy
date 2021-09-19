pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
    D8,
}

pub enum Instruction {
    ADD(ArithmeticTarget),
    ADDC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Instruction> {
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
            0xC6 => Some(Instruction::ADD(ArithmeticTarget::D8)),

            // ADDC
            0x88 => Some(Instruction::ADDC(ArithmeticTarget::B)),
            0x89 => Some(Instruction::ADDC(ArithmeticTarget::C)),
            0x8A => Some(Instruction::ADDC(ArithmeticTarget::D)),
            0x8B => Some(Instruction::ADDC(ArithmeticTarget::E)),
            0x8C => Some(Instruction::ADDC(ArithmeticTarget::H)),
            0x8D => Some(Instruction::ADDC(ArithmeticTarget::L)),
            0x8E => Some(Instruction::ADDC(ArithmeticTarget::HL)),
            0x8F => Some(Instruction::ADDC(ArithmeticTarget::A)),
            0xCE => Some(Instruction::ADDC(ArithmeticTarget::D8)),

            // SUB
            0x90 => Some(Instruction::SUB(ArithmeticTarget::B)),
            0x91 => Some(Instruction::SUB(ArithmeticTarget::C)),
            0x92 => Some(Instruction::SUB(ArithmeticTarget::D)),
            0x93 => Some(Instruction::SUB(ArithmeticTarget::E)),
            0x94 => Some(Instruction::SUB(ArithmeticTarget::H)),
            0x95 => Some(Instruction::SUB(ArithmeticTarget::L)),
            0x96 => Some(Instruction::SUB(ArithmeticTarget::HL)),
            0x97 => Some(Instruction::SUB(ArithmeticTarget::A)),
            0xD6 => Some(Instruction::SUB(ArithmeticTarget::D8)),

            // SBC
            0x98 => Some(Instruction::SBC(ArithmeticTarget::B)),
            0x99 => Some(Instruction::SBC(ArithmeticTarget::C)),
            0x9A => Some(Instruction::SBC(ArithmeticTarget::D)),
            0x9B => Some(Instruction::SBC(ArithmeticTarget::E)),
            0x9C => Some(Instruction::SBC(ArithmeticTarget::H)),
            0x9D => Some(Instruction::SBC(ArithmeticTarget::L)),
            0x9E => Some(Instruction::SBC(ArithmeticTarget::HL)),
            0x9F => Some(Instruction::SBC(ArithmeticTarget::A)),
            0xDE => Some(Instruction::SBC(ArithmeticTarget::D8)),

            _ => None,
        }
    }
}
