use std::convert;

const ZERO_BIT: u8 = 7;
const SUBSTRACTION_BIT: u8 = 6;
const HALF_CARRY_BIT: u8 = 5;
const CARRY_BIT: u8 = 4;

pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

struct FlagRegister {
    zero: bool,
    substraction: bool,
    half_carry: bool,
    carry: bool,
}

impl std::convert::From<FlagRegister> for u8 {
    fn from(flag: FlagRegister) -> u8 {
        (if flag.zero { 1 } else { 0 }) << ZERO_BIT
            | (if flag.substraction { 1 } else { 0 }) << SUBSTRACTION_BIT
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_BIT
            | (if flag.carry { 1 } else { 0 }) << CARRY_BIT
    }
}

impl std::convert::From<u8> for FlagRegister {
    fn from(byte: u8) -> FlagRegister {
        let zero = ((byte & 0x80) >> ZERO_BIT) != 0;
        let substraction = ((byte & 0x40) >> SUBSTRACTION_BIT) != 0;
        let half_carry = ((byte & 0x20) >> HALF_CARRY_BIT) != 0;
        let carry = ((byte & 0x10) >> CARRY_BIT) != 0;

        FlagRegister {
            zero: zero,
            substraction: substraction,
            half_carry: half_carry,
            carry: carry,
        }
    }
}

impl Registers {
    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc() {
        let mut regs = Registers {
            a: 10,
            b: 1,
            c: 3,
            d: 4,
            e: 5,
            f: 6,
            h: 7,
            l: 8,
        };
        regs.set_bc(0b0011_1000);
        assert_eq!(regs.get_bc(), 0b0011_1000);
    }

    #[test]
    fn test_af() {
        let mut regs = Registers {
            a: 10,
            b: 1,
            c: 3,
            d: 4,
            e: 5,
            f: 6,
            h: 7,
            l: 8,
        };
        regs.set_af(0b0010_1101);
        assert_eq!(regs.get_af(), 0b0010_1101);
    }

    #[test]
    fn test_de() {
        let mut regs = Registers {
            a: 10,
            b: 1,
            c: 3,
            d: 4,
            e: 5,
            f: 6,
            h: 7,
            l: 8,
        };
        regs.set_de(0b1010_1001);
        assert_eq!(regs.get_de(), 0b1010_1001);
    }

    #[test]
    fn test_flag() {
        let flag_byte = u8::from(FlagRegister {
            zero: true,
            substraction: false,
            half_carry: true,
            carry: true,
        });

        assert_eq!(flag_byte, 0b1011_0000);
    }

    #[test]
    fn test_flag_ex() {
        let flag_reg = FlagRegister::from(0b0101_0000);

        assert_eq!(flag_reg.zero, false);
        assert_eq!(flag_reg.substraction, true);
        assert_eq!(flag_reg.half_carry, false);
        assert_eq!(flag_reg.carry, true);
    }
}
