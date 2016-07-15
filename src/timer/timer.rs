use std::fmt;
use interrupt;

#[derive(Debug)]
enum InputClockFreq {
    Freq4096 = 4096,
    Freq262144 = 262144,
    Freq65536 = 65536,
    Freq16384 = 16384,
}

impl InputClockFreq {
    fn to_cycles(&self) -> u32 {
        use self::InputClockFreq::*;
        match *self {
            Freq4096 => 256,
            Freq262144 => 4,
            Freq65536 => 16,
            Freq16384 => 64,
        }
    }
}

pub struct Timer {
    counter: u8,
    modulo: u8,
    div: u8,
    idiv: u32,
    icounter: u32,
    enabled: bool,
    input_freq: InputClockFreq,
}

impl fmt::Debug for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Timer")
            .field("enabled", &self.enabled)
            .field("counter", &format_args!("0x{:02x}", self.counter))
            .field("modulo", &format_args!("0x{:02x}", self.modulo))
            .field("counter", &format_args!("0x{:02x}", self.counter))
            .field("input_freq", &self.input_freq)
            .finish()
    }
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            counter: 0,
            modulo: 0,
            div: 0,
            idiv: 0,
            icounter: 0,
            enabled: false,
            input_freq: InputClockFreq::Freq4096,
        }
    }

    pub fn get_div(&self) -> u8 {
        self.div
    }

    pub fn set_div(&mut self, val: u8) {
        self.div = 0;
    }

    pub fn get_tima(&self) -> u8 {
        self.counter
    }

    pub fn set_tima(&mut self, val: u8) {
        self.counter = val;
    }

    pub fn get_tma(&self) -> u8 {
        self.modulo
    }

    pub fn set_tma(&mut self, val: u8) {
        self.modulo = val;
    }

    pub fn get_tac(&self) -> u8 {
        use self::InputClockFreq::*;
        let ics = match self.input_freq {
            Freq4096 => 0x00,
            Freq262144 => 0x01,
            Freq65536 => 0x10,
            Freq16384 => 0x11,
        };
        (self.enabled as u8) << 2 | ics
    }

    pub fn set_tac(&mut self, val: u8) {
        use self::InputClockFreq::*;
        self.input_freq = match val >> 2 {
            0x00 => Freq4096,
            0x01 => Freq262144,
            0x10 => Freq65536,
            0x11 => Freq16384,
            _ => panic!(),
        };
        self.enabled = (val & 0x01) != 0;
    }

    pub fn step(&mut self, cycles: u32, ic: &mut interrupt::InterruptController) {
        self.step_div(cycles);

        if !self.enabled {
            return;
        }

        // TODO: Merge icounter and idiv into a single counter that div
        // can be derived from.
        self.icounter += cycles;
        if self.icounter >= self.input_freq.to_cycles() {
            if self.counter != 0xFF {
                self.counter = self.counter.wrapping_add(1);
            } else {
                self.counter = self.modulo;
                ic.request_interrupt(interrupt::Interrupt::Timer);
            }
        }
    }

    fn step_div(&mut self, cycles: u32) {
        self.idiv += cycles;
        // Every time the CPU clock does 256 cycles, increment the div reg.
        // 4Mhz / 256hz = 16Khz
        if self.idiv >= 0xFF {
            self.div = self.div.wrapping_add(1);
            self.idiv -= 0xFF;
        }
    }
}
