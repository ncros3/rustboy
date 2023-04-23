pub enum GameBoyKey {
    START,
    SELECT,
    B,
    A,
    DOWN,
    UP,
    LEFT,
    RIGHT,
}

pub struct Keypad {
    action_buttons: bool,
    direction_buttons: bool,
    // action buttons
    start: bool,
    select: bool,
    b: bool,
    a: bool,
    // direction buttons
    down: bool,
    up: bool,
    left: bool,
    right: bool,
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            action_buttons: false,
            direction_buttons: false,
            // action buttons
            start: false,
            select: false,
            b: false,
            a: false,
            // direction buttons
            down: false,
            up: false,
            left: false,
            right: false,
        }
    }

    pub fn control(&mut self, data: u8) {
        self.action_buttons = ((data >> 5) & 0x01) == 0;
        self.direction_buttons = ((data >> 4) & 0x01) == 0;
    }

    pub fn get(&self) -> u8 {
        match (self.action_buttons, self.direction_buttons) {
            (true, false) => {
                (!self.action_buttons as u8) << 5
                | (!self.direction_buttons as u8) << 4
                | (!self.start as u8) << 3
                | (!self.select as u8) << 2
                | (!self.b as u8) << 1
                | (!self.a as u8) << 0
            },
            (false, true) => {
                (!self.action_buttons as u8) << 5
                | (!self.direction_buttons as u8) << 4
                | (!self.down as u8) << 3
                | (!self.up as u8) << 2
                | (!self.left as u8) << 1
                | (!self.right as u8) << 0  
            },
            (false, false) => 0x00, // nothing to return
            (true, true) => panic!("Cannot read action and direction buttons at the same time"),
        }
    }

    pub fn set(&mut self, key: GameBoyKey, value: bool) {
        match key {
            GameBoyKey::START => self.start = value,
            GameBoyKey::SELECT => self.select = value,
            GameBoyKey::B => self.b = value,
            GameBoyKey::A => self.a = value,
            GameBoyKey::DOWN => self.down = value,
            GameBoyKey::UP => self.up = value,
            GameBoyKey::LEFT => self.left = value,
            GameBoyKey::RIGHT => self.right = value,
        }
    }
}

#[cfg(test)]
mod keypad_tests {
    use super::*;

    #[test]
    fn test_set_get_gameboykey() {
        let mut keypad = Keypad::new();

        keypad.control(0x10);
        keypad.set(GameBoyKey::START, true);
        assert_eq!(keypad.get(), 0x17);
        keypad.set(GameBoyKey::START, false);
        keypad.set(GameBoyKey::B, true);
        assert_eq!(keypad.get(), 0x1D);

        keypad.control(0x20);
        assert_eq!(keypad.get(), 0x2F);

        keypad.set(GameBoyKey::DOWN, false);
        keypad.set(GameBoyKey::UP, true);
        keypad.set(GameBoyKey::LEFT, false);
        keypad.set(GameBoyKey::RIGHT, true);
        assert_eq!(keypad.get(), 0x2A);

        keypad.set(GameBoyKey::DOWN, true);
        keypad.set(GameBoyKey::UP, false);
        keypad.set(GameBoyKey::LEFT, true);
        keypad.set(GameBoyKey::RIGHT, false);
        assert_eq!(keypad.get(), 0x25);
    }
}