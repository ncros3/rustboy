const ZERO_BIT: u8 = 7;
const SUBSTRACTION_BIT: u8 = 6;
const HALF_CARRY_BIT: u8 = 5;
const CARRY_BIT: u8 = 4;

#[derive(Copy, Clone)]
struct FlagRegister {
    zero: bool,
    substraction: bool,
    half_carry: bool,
    carry: bool,
}

impl FlagRegister {
    fn new() -> FlagRegister {
        FlagRegister {
            zero: false,
            substraction: false,
            half_carry: false,
            carry: false,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagRegister,
    h: u8,
    l: u8,
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
    fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagRegister::new(),
            h: 0,
            l: 0,
        }
    }

    fn read_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn write_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    fn read_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f) as u16)
    }

    fn write_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = FlagRegister::from(value as u8);
    }

    fn read_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn write_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    fn read_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn write_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc() {
        let mut regs = Registers::new();
        regs.write_bc(0b0011_1000_1010_0110);
        assert_eq!(regs.read_bc(), 0b0011_1000_1010_0110);
    }

    #[test]
    fn test_af() {
        let mut regs = Registers::new();
        regs.write_af(0b0011_1000_1010_0000);
        assert_eq!(regs.read_af(), 0b0011_1000_1010_0000);
    }

    #[test]
    fn test_de() {
        let mut regs = Registers::new();
        regs.write_de(0b0011_1000_1010_0110);
        assert_eq!(regs.read_de(), 0b0011_1000_1010_0110);
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
