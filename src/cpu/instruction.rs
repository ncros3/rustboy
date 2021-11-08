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

pub enum IncDecTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

pub enum U16Target {
    BC,
    DE,
    HL,
    SP,
}

pub enum Load16Target {
    BC,
    DE,
    HL_plus,
    HL_minus,
}

pub enum JumpTarget {
    IMMEDIATE,
    NZ,
    NC,
    Z,
    C,
}

pub enum SPTarget {
    FROM_SP,
    TO_HL,
    TO_SP,
}

pub enum RamTarget {
    OneByteAddress,
    AddressFromRegister,
    TwoBytesAddress,
}

pub enum PopPushTarget {
    BC,
    DE,
    HL,
    AF,
}

pub enum ResetTarget {
    FLASH_0,
    FLASH_1,
    FLASH_2,
    FLASH_3,
    FLASH_4,
    FLASH_5,
    FLASH_6,
    FLASH_7,
}

pub enum Instruction {
    ADD(ArithmeticTarget),
    ADDC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
    AND(ArithmeticTarget),
    OR(ArithmeticTarget),
    XOR(ArithmeticTarget),
    CP(ArithmeticTarget),
    INC(IncDecTarget),
    DEC(IncDecTarget),
    INC16(U16Target),
    DEC16(U16Target),
    ADD16(U16Target),
    LOAD(IncDecTarget, ArithmeticTarget),
    LOAD_INDIRECT(Load16Target),
    LOAD_IMMEDIATE(U16Target),
    STORE_INDIRECT(Load16Target),
    LOAD_SP(SPTarget),
    LOAD_RAM(RamTarget),
    STORE_RAM(RamTarget),
    JUMP_RELATIVE(JumpTarget),
    JUMP_IMMEDIATE(JumpTarget),
    JUMP_INDIRECT,
    RETURN(JumpTarget),
    RESET(ResetTarget),
    CALL(JumpTarget),
    POP(PopPushTarget),
    PUSH(PopPushTarget),
    AddSp,
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

            // ADD 16 bits
            0x09 => Some(Instruction::ADD16(U16Target::BC)),
            0x19 => Some(Instruction::ADD16(U16Target::DE)),
            0x29 => Some(Instruction::ADD16(U16Target::HL)),
            0x39 => Some(Instruction::ADD16(U16Target::SP)),

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

            // ADD Stack pointer
            0xE8 => Some(Instruction::AddSp),

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

            // AND
            0xA0 => Some(Instruction::AND(ArithmeticTarget::B)),
            0xA1 => Some(Instruction::AND(ArithmeticTarget::C)),
            0xA2 => Some(Instruction::AND(ArithmeticTarget::D)),
            0xA3 => Some(Instruction::AND(ArithmeticTarget::E)),
            0xA4 => Some(Instruction::AND(ArithmeticTarget::H)),
            0xA5 => Some(Instruction::AND(ArithmeticTarget::L)),
            0xA6 => Some(Instruction::AND(ArithmeticTarget::HL)),
            0xA7 => Some(Instruction::AND(ArithmeticTarget::A)),
            0xE6 => Some(Instruction::AND(ArithmeticTarget::D8)),

            // XOR
            0xA8 => Some(Instruction::XOR(ArithmeticTarget::B)),
            0xA9 => Some(Instruction::XOR(ArithmeticTarget::C)),
            0xAA => Some(Instruction::XOR(ArithmeticTarget::D)),
            0xAB => Some(Instruction::XOR(ArithmeticTarget::E)),
            0xAC => Some(Instruction::XOR(ArithmeticTarget::H)),
            0xAD => Some(Instruction::XOR(ArithmeticTarget::L)),
            0xAE => Some(Instruction::XOR(ArithmeticTarget::HL)),
            0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),
            0xEE => Some(Instruction::XOR(ArithmeticTarget::D8)),

            // OR
            0xB0 => Some(Instruction::OR(ArithmeticTarget::B)),
            0xB1 => Some(Instruction::OR(ArithmeticTarget::C)),
            0xB2 => Some(Instruction::OR(ArithmeticTarget::D)),
            0xB3 => Some(Instruction::OR(ArithmeticTarget::E)),
            0xB4 => Some(Instruction::OR(ArithmeticTarget::H)),
            0xB5 => Some(Instruction::OR(ArithmeticTarget::L)),
            0xB6 => Some(Instruction::OR(ArithmeticTarget::HL)),
            0xB7 => Some(Instruction::OR(ArithmeticTarget::A)),
            0xF6 => Some(Instruction::OR(ArithmeticTarget::D8)),

            // CP
            0xB8 => Some(Instruction::CP(ArithmeticTarget::B)),
            0xB9 => Some(Instruction::CP(ArithmeticTarget::C)),
            0xBA => Some(Instruction::CP(ArithmeticTarget::D)),
            0xBB => Some(Instruction::CP(ArithmeticTarget::E)),
            0xBC => Some(Instruction::CP(ArithmeticTarget::H)),
            0xBD => Some(Instruction::CP(ArithmeticTarget::L)),
            0xBE => Some(Instruction::CP(ArithmeticTarget::HL)),
            0xBF => Some(Instruction::CP(ArithmeticTarget::A)),
            0xFE => Some(Instruction::CP(ArithmeticTarget::D8)),

            // INC
            0x04 => Some(Instruction::INC(IncDecTarget::B)),
            0x0C => Some(Instruction::INC(IncDecTarget::C)),
            0x14 => Some(Instruction::INC(IncDecTarget::D)),
            0x1C => Some(Instruction::INC(IncDecTarget::E)),
            0x24 => Some(Instruction::INC(IncDecTarget::H)),
            0x2C => Some(Instruction::INC(IncDecTarget::L)),
            0x34 => Some(Instruction::INC(IncDecTarget::HL)),
            0x3C => Some(Instruction::INC(IncDecTarget::A)),

            // INC 16 bits
            0x03 => Some(Instruction::INC16(U16Target::BC)),
            0x13 => Some(Instruction::INC16(U16Target::DE)),
            0x23 => Some(Instruction::INC16(U16Target::HL)),
            0x33 => Some(Instruction::INC16(U16Target::SP)),

            // DEC
            0x05 => Some(Instruction::DEC(IncDecTarget::B)),
            0x0D => Some(Instruction::DEC(IncDecTarget::C)),
            0x15 => Some(Instruction::DEC(IncDecTarget::D)),
            0x1D => Some(Instruction::DEC(IncDecTarget::E)),
            0x25 => Some(Instruction::DEC(IncDecTarget::H)),
            0x2D => Some(Instruction::DEC(IncDecTarget::L)),
            0x35 => Some(Instruction::DEC(IncDecTarget::HL)),
            0x3D => Some(Instruction::DEC(IncDecTarget::A)),

            // DEC 16 bits
            0x0B => Some(Instruction::DEC16(U16Target::BC)),
            0x1B => Some(Instruction::DEC16(U16Target::DE)),
            0x2B => Some(Instruction::DEC16(U16Target::HL)),
            0x3B => Some(Instruction::DEC16(U16Target::SP)),

            // LOAD instruction
            0x40 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::B)),
            0x41 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::C)),
            0x42 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::D)),
            0x43 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::E)),
            0x44 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::H)),
            0x45 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::L)),
            0x46 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::HL)),
            0x47 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::A)),
            0x06 => Some(Instruction::LOAD(IncDecTarget::B, ArithmeticTarget::D8)),

            0x48 => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::B)),
            0x49 => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::C)),
            0x4A => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::D)),
            0x4B => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::E)),
            0x4C => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::H)),
            0x4D => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::L)),
            0x4E => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::HL)),
            0x4F => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::A)),
            0x0E => Some(Instruction::LOAD(IncDecTarget::C, ArithmeticTarget::D8)),

            0x50 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::B)),
            0x51 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::C)),
            0x52 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::D)),
            0x53 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::E)),
            0x54 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::H)),
            0x55 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::L)),
            0x56 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::HL)),
            0x57 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::A)),
            0x16 => Some(Instruction::LOAD(IncDecTarget::D, ArithmeticTarget::D8)),

            0x58 => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::B)),
            0x59 => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::C)),
            0x5A => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::D)),
            0x5B => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::E)),
            0x5C => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::H)),
            0x5D => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::L)),
            0x5E => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::HL)),
            0x5F => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::A)),
            0x1E => Some(Instruction::LOAD(IncDecTarget::E, ArithmeticTarget::D8)),

            0x60 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::B)),
            0x61 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::C)),
            0x62 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::D)),
            0x63 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::E)),
            0x64 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::H)),
            0x65 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::L)),
            0x66 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::HL)),
            0x67 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::A)),
            0x26 => Some(Instruction::LOAD(IncDecTarget::H, ArithmeticTarget::D8)),

            0x68 => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::B)),
            0x69 => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::C)),
            0x6A => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::D)),
            0x6B => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::E)),
            0x6C => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::H)),
            0x6D => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::L)),
            0x6E => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::HL)),
            0x6F => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::A)),
            0x2E => Some(Instruction::LOAD(IncDecTarget::L, ArithmeticTarget::D8)),

            0x70 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::B)),
            0x71 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::C)),
            0x72 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::D)),
            0x73 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::E)),
            0x74 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::H)),
            0x75 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::L)),
            0x77 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::A)),
            0x36 => Some(Instruction::LOAD(IncDecTarget::HL, ArithmeticTarget::D8)),

            0x78 => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::B)),
            0x79 => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::C)),
            0x7A => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::D)),
            0x7B => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::E)),
            0x7C => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::H)),
            0x7D => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::L)),
            0x7E => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::HL)),
            0x3E => Some(Instruction::LOAD(IncDecTarget::A, ArithmeticTarget::D8)),

            0x0A => Some(Instruction::LOAD_INDIRECT(Load16Target::BC)),
            0x1A => Some(Instruction::LOAD_INDIRECT(Load16Target::DE)),
            0x2A => Some(Instruction::LOAD_INDIRECT(Load16Target::HL_plus)),
            0x3A => Some(Instruction::LOAD_INDIRECT(Load16Target::HL_minus)),

            0x01 => Some(Instruction::LOAD_IMMEDIATE(U16Target::BC)),
            0x11 => Some(Instruction::LOAD_IMMEDIATE(U16Target::DE)),
            0x21 => Some(Instruction::LOAD_IMMEDIATE(U16Target::HL)),
            0x31 => Some(Instruction::LOAD_IMMEDIATE(U16Target::SP)),

            0x02 => Some(Instruction::STORE_INDIRECT(Load16Target::BC)),
            0x12 => Some(Instruction::STORE_INDIRECT(Load16Target::DE)),
            0x22 => Some(Instruction::STORE_INDIRECT(Load16Target::HL_plus)),
            0x32 => Some(Instruction::STORE_INDIRECT(Load16Target::HL_minus)),

            0x08 => Some(Instruction::LOAD_SP(SPTarget::FROM_SP)),
            0xF8 => Some(Instruction::LOAD_SP(SPTarget::TO_HL)),
            0xF9 => Some(Instruction::LOAD_SP(SPTarget::TO_SP)),

            0xF0 => Some(Instruction::LOAD_RAM(RamTarget::OneByteAddress)),
            0xF2 => Some(Instruction::LOAD_RAM(RamTarget::AddressFromRegister)),
            0xFA => Some(Instruction::LOAD_RAM(RamTarget::TwoBytesAddress)),

            0xE0 => Some(Instruction::STORE_RAM(RamTarget::OneByteAddress)),
            0xE2 => Some(Instruction::STORE_RAM(RamTarget::AddressFromRegister)),
            0xEA => Some(Instruction::STORE_RAM(RamTarget::TwoBytesAddress)),

            // JUMP instructions
            0x20 => Some(Instruction::JUMP_RELATIVE(JumpTarget::NZ)),
            0x30 => Some(Instruction::JUMP_RELATIVE(JumpTarget::NC)),
            0x18 => Some(Instruction::JUMP_RELATIVE(JumpTarget::IMMEDIATE)),
            0x28 => Some(Instruction::JUMP_RELATIVE(JumpTarget::Z)),
            0x38 => Some(Instruction::JUMP_RELATIVE(JumpTarget::C)),

            0xC2 => Some(Instruction::JUMP_IMMEDIATE(JumpTarget::NZ)),
            0xD2 => Some(Instruction::JUMP_IMMEDIATE(JumpTarget::NC)),
            0xC3 => Some(Instruction::JUMP_IMMEDIATE(JumpTarget::IMMEDIATE)),
            0xCA => Some(Instruction::JUMP_IMMEDIATE(JumpTarget::Z)),
            0xDA => Some(Instruction::JUMP_IMMEDIATE(JumpTarget::C)),

            0xE9 => Some(Instruction::JUMP_INDIRECT),

            // RETURN instructions
            0xC0 => Some(Instruction::RETURN(JumpTarget::NZ)),
            0xD0 => Some(Instruction::RETURN(JumpTarget::NC)),
            0xC8 => Some(Instruction::RETURN(JumpTarget::Z)),
            0xD8 => Some(Instruction::RETURN(JumpTarget::C)),
            0xC9 => Some(Instruction::RETURN(JumpTarget::IMMEDIATE)),

            // RESET instructions
            0xC7 => Some(Instruction::RESET(ResetTarget::FLASH_0)),
            0xCF => Some(Instruction::RESET(ResetTarget::FLASH_1)),
            0xD7 => Some(Instruction::RESET(ResetTarget::FLASH_2)),
            0xDF => Some(Instruction::RESET(ResetTarget::FLASH_3)),
            0xE7 => Some(Instruction::RESET(ResetTarget::FLASH_4)),
            0xEF => Some(Instruction::RESET(ResetTarget::FLASH_5)),
            0xF7 => Some(Instruction::RESET(ResetTarget::FLASH_6)),
            0xFF => Some(Instruction::RESET(ResetTarget::FLASH_7)),

            // CALL instructions
            0xC4 => Some(Instruction::CALL(JumpTarget::NZ)),
            0xD4 => Some(Instruction::CALL(JumpTarget::NC)),
            0xCC => Some(Instruction::CALL(JumpTarget::Z)),
            0xDC => Some(Instruction::CALL(JumpTarget::C)),
            0xCD => Some(Instruction::CALL(JumpTarget::IMMEDIATE)),

            // POP & PUSH instructions
            0xC1 => Some(Instruction::POP(PopPushTarget::BC)),
            0xD1 => Some(Instruction::POP(PopPushTarget::DE)),
            0xE1 => Some(Instruction::POP(PopPushTarget::HL)),
            0xF1 => Some(Instruction::POP(PopPushTarget::AF)),

            0xC5 => Some(Instruction::PUSH(PopPushTarget::BC)),
            0xD5 => Some(Instruction::PUSH(PopPushTarget::DE)),
            0xE5 => Some(Instruction::PUSH(PopPushTarget::HL)),
            0xF5 => Some(Instruction::PUSH(PopPushTarget::AF)),

            _ => None,
        }
    }
}
