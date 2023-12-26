use std::fmt;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub bytes: Vec<u8>,
    pub bitpos: usize,
    pub bytepos: usize,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(by:{}, bi:{}) [", self.bytepos, self.bitpos)?;
        for byte in self.bytes.iter() {
            write!(f, "{:08b}, ", byte)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl Symbol {
    pub fn append_bit(&mut self, bit: bool) {
        if self.bytes.is_empty() {
            self.bytes.push(0);
        }
        if self.bitpos == 8 {
            self.inc_size();
        }
        if bit {
            self.bytes[self.bytepos] |= 128 >> self.bitpos;
        }
        self.bitpos += 1;
    }

    fn inc_size(&mut self) {
        self.bytes.push(0);
        self.bitpos = 0;
        self.bytepos += 1;
    }

    //only use when self.bytes already has at least one u8 in it
    fn append_byte(&mut self, mut byte: u8, mut bits: usize) {
        while bits > 0 {
            if self.bitpos == 8 {
                self.inc_size();
            }
            let written_bytes = 8 - self.bitpos;
            self.bytes[self.bytepos] |= byte >> self.bitpos;
            if written_bytes < bits {
                //current byte is full but bits are left
                byte <<= written_bytes;
                bits -= written_bytes;
                self.inc_size();
            } else {
                //all bits written
                self.bitpos += bits;
                return;
            }
        }
    }

    pub fn append_sym(&mut self, sym: &Symbol) {
        let mut byteindex = 0;
        if self.bytes.is_empty() {
            self.bytes.push(0);
        }
        while byteindex < sym.bytepos {
            self.append_byte(sym.bytes[byteindex], 8);
            byteindex += 1;
        }
        if sym.bitpos > 0 {
            self.append_byte(sym.bytes[byteindex], sym.bitpos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_less_to_empty() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11010000],
            bitpos: 4,
            bytepos: 0,
        };
        empty.append_sym(&small);
        assert_eq!(vec![0b11010000], empty.bytes);
        assert_eq!(4, empty.bitpos);
        assert_eq!(0, empty.bytepos);
    }

    #[test]
    fn add_eight_to_empty() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11000001],
            bitpos: 8,
            bytepos: 0,
        };
        empty.append_sym(&small);
        assert_eq!(vec![0b11000001], empty.bytes);
        assert_eq!(8, empty.bitpos);
        assert_eq!(0, empty.bytepos);
    }

    #[test]
    fn add_nine_to_empty() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11000001, 0b10000000],
            bitpos: 1,
            bytepos: 1,
        };
        empty.append_sym(&small);
        assert_eq!(vec![0b11000001, 0b10000000], empty.bytes);
        assert_eq!(1, empty.bitpos);
        assert_eq!(1, empty.bytepos);
    }

    #[test]
    fn add_two_fitting() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11000010],
            bitpos: 7,
            bytepos: 0,
        };
        let small2 = Symbol {
            bytes: vec![0b10000000],
            bitpos: 1,
            bytepos: 0,
        };
        empty.append_sym(&small);
        empty.append_sym(&small2);
        assert_eq!(vec![0b11000011], empty.bytes);
        assert_eq!(8, empty.bitpos);
        assert_eq!(0, empty.bytepos);
    }

    #[test]
    fn add_two_2nd_no_fit() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11000010],
            bitpos: 7,
            bytepos: 0,
        };
        let small2 = Symbol {
            bytes: vec![0b10100000],
            bitpos: 3,
            bytepos: 0,
        };
        empty.append_sym(&small);
        empty.append_sym(&small2);
        assert_eq!(vec![0b11000011, 0b01000000], empty.bytes);
        assert_eq!(2, empty.bitpos);
        assert_eq!(1, empty.bytepos);
    }

    #[test]
    fn add_two_1st_no_fit() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11000000, 0b10000000],
            bitpos: 1,
            bytepos: 1,
        };
        let small2 = Symbol {
            bytes: vec![0b10100000],
            bitpos: 3,
            bytepos: 0,
        };
        empty.append_sym(&small);
        empty.append_sym(&small2);
        assert_eq!(vec![0b11000000, 0b11010000], empty.bytes);
        assert_eq!(4, empty.bitpos);
        assert_eq!(1, empty.bytepos);
    }

    #[test]
    fn add_two_both_no_fit() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        let small = Symbol {
            bytes: vec![0b11000000, 0b10000000],
            bitpos: 1,
            bytepos: 1,
        };
        let small2 = Symbol {
            bytes: vec![0b11000000, 0b00000001],
            bitpos: 8,
            bytepos: 1,
        };
        empty.append_sym(&small);
        empty.append_sym(&small2);
        assert_eq!(
            vec![0b11000000, 0b11100000, 0b00000000, 0b10000000],
            empty.bytes
        );
        assert_eq!(1, empty.bitpos);
        assert_eq!(3, empty.bytepos);
    }

    #[test]
    fn add_bit_inside() {
        let mut empty = Symbol {
            bytes: Vec::new(),
            bitpos: 0,
            bytepos: 0,
        };
        empty.append_bit(false);
        assert_eq!(vec![0b00000000], empty.bytes);
        assert_eq!(1, empty.bitpos);
        assert_eq!(0, empty.bytepos);
    }

    #[test]
    fn add_bit_over_border() {
        let mut empty = Symbol {
            bytes: vec![0],
            bitpos: 8,
            bytepos: 0,
        };
        empty.append_bit(true);
        assert_eq!(vec![0b00000000, 0b10000000], empty.bytes);
        assert_eq!(1, empty.bitpos);
        assert_eq!(1, empty.bytepos);
    }
}
