use std::fmt;

#[derive(Clone, Copy)]
pub enum Interrupt {
    VBlank = 1,
    LCDCStat = 1 << 1,
    Timer = 1 << 2,
    Serial = 1 << 3,
    Joypad = 1 << 4,
}

impl Interrupt {
    pub fn get_addr(&self) -> u16 {
        use self::Interrupt::*;
        match *self {
            VBlank => 0x0040,
            LCDCStat => 0x0048,
            Timer => 0x0050,
            Serial => 0x0058,
            Joypad => 0x0060,
        }
    }
}

pub struct InterruptController {
    pub ime: bool,
    pub iflag: u8,
    pub ie: u8,
}

impl fmt::Debug for InterruptController {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,
                 "int ctrl: ie: 0x{:08b} iflag 0x{:08b}",
                 self.ie,
                 self.iflag)
    }
}

impl InterruptController {
    pub fn new() -> InterruptController {
        InterruptController {
            ime: false,
            iflag: 0,
            ie: 0,
        }
    }

    pub fn get_interrupt(&self) -> Option<Interrupt> {
        use self::Interrupt::*;
        let interrupt = self.iflag & self.ie;

        if interrupt == 0x0 || !self.ime {
            None
        } else if (interrupt & 1) == 1 {
            Some(VBlank)
        } else if (interrupt & (1 << 1)) == 1 << 1 {
            Some(LCDCStat)
        } else if (interrupt & (1 << 2)) == 1 << 2 {
            Some(Timer)
        } else if (interrupt & (1 << 3)) == 1 << 3 {
            Some(Serial)
        } else if (interrupt & (1 << 4)) == 1 << 4 {
            Some(Joypad)
        } else {
            panic!("Invalid interrupt! IE: {:x} IF {:x}", self.ie, self.iflag);
        }
    }

    pub fn reset_interrupt(&mut self, int: Interrupt) {
        self.iflag &= !(int as u8);
    }

    pub fn request_interrupt(&mut self, int: Interrupt) {
        self.iflag |= int as u8;
    }
}
