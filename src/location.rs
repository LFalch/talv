use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Letter(u8);

impl Letter {
    pub fn new(i: u8) -> Option<Self> {
        if i < 8 {
            Some(Letter(i))
        } else { None }
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
    pub fn new(i: u8) -> Option<Self> {
        if i < 8 {
            Some(Number(i << 4))
        } else { None }
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            .and_then(|l| {
                Number::from_i8(self.n().i8() + n)
                    .map(|n| Coords::new(l, n))
            })
    }
    pub fn i8_tuple(self) -> (i8, i8) {
        (self.l().i8(), self.n().i8())
    }
    pub fn from_u8_tuple(l: i8, n: i8) -> Option<Self> {
        Some(Coords::new(Letter::new(l as u8)?, Number::new(n as u8)?))
    }
    pub fn sub(self, other: Self) -> (i8, i8) {
        (self.l().i8()-other.l().i8(), self.n().i8()-other.n().i8())
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
    (2, 1), (2, -1),
    (1, 2), (1, -2),
    (-2, 1), (-2, -1),
    (-1, 2), (-1, -2),
];
