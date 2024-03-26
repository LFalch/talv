use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct File(u8);

impl File {
    pub const A: Self = File(0);
    pub const B: Self = File(1);
    pub const C: Self = File(2);
    pub const D: Self = File(3);
    pub const E: Self = File(4);
    pub const F: Self = File(5);
    pub const G: Self = File(6);
    pub const H: Self = File(7);

    pub const fn new(i: u8) -> Option<Self> {
        if i < 8 {
            Some(File(i))
        } else {
            None
        }
    }
    pub fn from_char(c: char) -> Option<Self> {
        if c.is_ascii_alphabetic() {
            let mut c = c;
            c.make_ascii_lowercase();

            File::new((c as u8).wrapping_sub(b'a'))
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

impl From<File> for usize {
    fn from(l: File) -> Self {
        l.i8() as usize
    }
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = (b'a' + self.0) as char;
        c.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(u8);

impl Rank {
    pub const N1: Self = Rank(0 << 3);
    pub const N2: Self = Rank(1 << 3);
    pub const N3: Self = Rank(2 << 3);
    pub const N4: Self = Rank(3 << 3);
    pub const N5: Self = Rank(4 << 3);
    pub const N6: Self = Rank(5 << 3);
    pub const N7: Self = Rank(6 << 3);
    pub const N8: Self = Rank(7 << 3);

    pub const fn new(i: u8) -> Option<Self> {
        if i < 8 {
            Some(Rank(i << 3))
        } else {
            None
        }
    }
    pub fn from_char(c: char) -> Option<Self> {
        if c.is_ascii() {
            Rank::new((c as u8).wrapping_sub(b'1'))
        } else {
            None
        }
    }
    pub fn from_i8(i: i8) -> Option<Self> {
        Self::new(i as u8)
    }
    pub fn i8(self) -> i8 {
        (self.0 >> 3) as i8
    }
}

impl From<Rank> for usize {
    fn from(l: Rank) -> Self {
        l.i8() as usize
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = (self.0 >> 3) + 1;
        n.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Coords(u8);

impl Coords {
    pub fn new(l: File, n: Rank) -> Self {
        Coords(l.0 | n.0)
    }
    pub fn from_str(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let l = File::from_char(chars.next()?)?;
        let n = Rank::from_char(chars.next()?)?;
        if chars.next().is_none() {
            Some(Self::new(l, n))
        } else {
            None
        }
    }
    pub fn f(self) -> File {
        File(self.0 & 0b111)
    }
    pub fn r(self) -> Rank {
        Rank(self.0 & 0b111_000)
    }
    /// Calculates a new location based on the relative
    /// coordinates given. Yields `None` if the resulting
    /// location is out of bounds.
    pub fn add(self, l: i8, n: i8) -> Option<Coords> {
        File::from_i8(self.f().i8() + l)
            .and_then(|l| Rank::from_i8(self.r().i8() + n).map(|n| Coords::new(l, n)))
    }
    pub fn i8_tuple(self) -> (i8, i8) {
        (self.f().i8(), self.r().i8())
    }
    pub fn from_u8_tuple(l: i8, n: i8) -> Option<Self> {
        Some(Coords::new(File::new(l as u8)?, Rank::new(n as u8)?))
    }
    pub fn sub(self, other: Self) -> (i8, i8) {
        (
            self.f().i8() - other.f().i8(),
            self.r().i8() - other.r().i8(),
        )
    }
    pub fn into_u8(self) -> u8 {
        self.0
    }
}

impl Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.f(), self.r())
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

pub struct RankRange {
    start: Rank,
    end: Rank,
}

impl RankRange {
    pub const fn full() -> Self {
        RankRange {
            start: Rank(0),
            end: Rank(8 << 3),
        }
    }
}

impl Iterator for RankRange {
    type Item = Rank;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let ret = self.start;
            self.start.0 += 0b1000;
            Some(ret)
        } else {
            None
        }
    }
}
impl DoubleEndedIterator for RankRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end.0 -= 0b1000;
            Some(self.end)
        } else {
            None
        }
    }
}

pub struct FileRange {
    start: File,
    end: File,
}

impl FileRange {
    pub const fn full() -> Self {
        FileRange {
            start: File(0),
            end: File(8),
        }
    }
}

impl Iterator for FileRange {
    type Item = File;
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
