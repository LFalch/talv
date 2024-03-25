use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Letter(u8);

impl Letter {
    pub const A: Self = Letter(0);
    pub const B: Self = Letter(1);
    pub const C: Self = Letter(2);
    pub const D: Self = Letter(3);
    pub const E: Self = Letter(4);
    pub const F: Self = Letter(5);
    pub const G: Self = Letter(6);
    pub const H: Self = Letter(7);

    pub const fn new(i: u8) -> Option<Self> {
        if i < 8 {
            Some(Letter(i))
        } else {
            None
        }
    }
    pub fn from_char(c: char) -> Option<Self> {
        if c.is_ascii_alphabetic() {
            let mut c = c;
            c.make_ascii_lowercase();

            Letter::new((c as u8).wrapping_sub(b'a'))
        } else {
            None
        }
    }
    pub fn from_i8(i: i8) -> Option<Self> {
        Self::new(i as u8)
    }
    pub fn i8(self) -> i8 {
        self.0 as i8
    }
}

impl From<Letter> for usize {
    fn from(l: Letter) -> Self {
        l.i8() as usize
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = (b'a' + self.0) as char;
        c.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Number(u8);

impl Number {
    pub const N1: Self = Number(0 << 4);
    pub const N2: Self = Number(1 << 4);
    pub const N3: Self = Number(2 << 4);
    pub const N4: Self = Number(3 << 4);
    pub const N5: Self = Number(4 << 4);
    pub const N6: Self = Number(5 << 4);
    pub const N7: Self = Number(6 << 4);
    pub const N8: Self = Number(7 << 4);

    pub const fn new(i: u8) -> Option<Self> {
        if i < 8 {
            Some(Number(i << 4))
        } else {
            None
        }
    }
    pub fn from_char(c: char) -> Option<Self> {
        if c.is_ascii() {
            Number::new((c as u8).wrapping_sub(b'1'))
        } else {
            None
        }
    }
    pub fn from_i8(i: i8) -> Option<Self> {
        Self::new(i as u8)
    }
    pub fn i8(self) -> i8 {
        (self.0 >> 4) as i8
    }
}

impl From<Number> for usize {
    fn from(l: Number) -> Self {
        l.i8() as usize
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = (self.0 >> 4) + 1;
        n.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coords(u8);

impl Coords {
    pub fn new(l: Letter, n: Number) -> Self {
        Coords(l.0 | n.0)
    }
    pub fn from_str(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let l = Letter::from_char(chars.next()?)?;
        let n = Number::from_char(chars.next()?)?;
        if chars.next().is_none() {
            Some(Self::new(l, n))
        } else {
            None
        }
    }
    pub fn l(self) -> Letter {
        Letter(self.0 & 0b1111)
    }
    pub fn n(self) -> Number {
        Number(self.0 & 0b1111_0000)
    }
    /// Calculates a new location based on the relative
    /// coordinates given. Yields `None` if the resulting
    /// location is out of bounds.
    pub fn add(self, l: i8, n: i8) -> Option<Coords> {
        Letter::from_i8(self.l().i8() + l)
            .and_then(|l| Number::from_i8(self.n().i8() + n).map(|n| Coords::new(l, n)))
    }
    pub fn i8_tuple(self) -> (i8, i8) {
        (self.l().i8(), self.n().i8())
    }
    pub fn from_u8_tuple(l: i8, n: i8) -> Option<Self> {
        Some(Coords::new(Letter::new(l as u8)?, Number::new(n as u8)?))
    }
    pub fn sub(self, other: Self) -> (i8, i8) {
        (
            self.l().i8() - other.l().i8(),
            self.n().i8() - other.n().i8(),
        )
    }
    /// number, letter
    pub fn indices(self) -> (usize, usize) {
        (self.n().into(), self.l().into())
    }
}

impl Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.l(), self.n())
    }
}

pub const LEAPS: [(i8, i8); 8] = [
    (2, 1),
    (2, -1),
    (1, 2),
    (1, -2),
    (-2, 1),
    (-2, -1),
    (-1, 2),
    (-1, -2),
];

pub struct NumberRange {
    start: Number,
    end: Number,
}

impl NumberRange {
    pub const fn full() -> Self {
        NumberRange {
            start: Number(0),
            end: Number(8 << 4),
        }
    }
}

impl Iterator for NumberRange {
    type Item = Number;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let ret = self.start;
            self.start.0 += 0b1_0000;
            Some(ret)
        } else {
            None
        }
    }
}
impl DoubleEndedIterator for NumberRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end.0 -= 0b1_0000;
            Some(self.end)
        } else {
            None
        }
    }
}

pub struct LetterRange {
    start: Letter,
    end: Letter,
}

impl LetterRange {
    pub const fn full() -> Self {
        LetterRange {
            start: Letter(0),
            end: Letter(8),
        }
    }
}

impl Iterator for LetterRange {
    type Item = Letter;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let ret = self.start;
            self.start.0 += 1;
            Some(ret)
        } else {
            None
        }
    }
}
