pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8
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
            l: 8
        };
        regs.set_bc(564);
        assert_eq!(regs.get_bc(), 564);
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
            l: 8
        };
        regs.set_af(623);
        assert_eq!(regs.get_af(), 623);
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
            l: 8
        };
        regs.set_de(754);
        assert_eq!(regs.get_de(), 754);
    }
}