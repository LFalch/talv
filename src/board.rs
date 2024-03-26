use std::{
    fmt::{self, Display}, ops::Not
};

use crate::location::Coords;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Piece {
    Pawn = 1,
    Rook = 2,
    Knight = 3,
    Bishop = 4,
    Queen = 5,
    King = 6,
}

impl Piece {
    #[inline]
    const fn from_u8(n: u8) -> Self {
        match n {
            1 => Self::Pawn,
            2 => Self::Rook,
            3 => Self::Knight,
            4 => Self::Bishop,
            5 => Self::Queen,
            6 => Self::King,
            _ => unreachable!()
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Piece::*;
        write!(
            f,
            "{}",
            match *self {
                Pawn => "",
                Rook => "R",
                Knight => "N",
                Bishop => "B",
                Queen => "Q",
                King => "K",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Colour {
    White,
    Black,
}

impl Not for Colour {
    type Output = Self;
    fn not(self) -> Self::Output {
        use self::Colour::*;
        match self {
            White => Black,
            Black => White,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Field {
    Empty,
    Occupied(Colour, Piece),
}

impl Field {
    pub fn is_empty(&self) -> bool {
        matches!(*self, Self::Empty)
    }
    pub fn is_occupied(&self) -> bool {
        matches!(*self, Self::Occupied(_, _))
    }
    pub fn into_piece(self) -> Option<Piece> {
        match self {
            Self::Empty => None,
            Self::Occupied(_, p) => Some(p),
        }
    }
    #[inline]
    const fn into_bits(self) -> u8 {
        match self {
            Field::Empty => 0,
            Field::Occupied(Colour::White, p) => p as u8,
            Field::Occupied(Colour::Black, p) => 0b1000 | p as u8,
        }
    }
    #[inline]
    const fn from_bits(n: u8) -> Self {
        let p = n & 0b111;
        if p == 0 {
            return Field::Empty;
        }
        let p = Piece::from_u8(p);
        let c = if n & 0b1000 == 0 {
            Colour::White
        } else {
            Colour::Black
        };

        Field::Occupied(c, p)
    }
    const fn or(self, other: Self) -> u8 {
        self.into_bits() | (other.into_bits() << 4)
    }
}

// 0b0000 nothing
// 0b_001 pawn
// 0b_010 rook
// 0b_011 knight
// 0b_100 bishop
// 0b_101 queen
// 0b_110 king
// 0b_111 INVALID
// 0b1___ INVALID

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Colour::*;
        use self::Field::*;
        use self::Piece::*;
        write!(
            f,
            "{}",
            match *self {
                Empty => " ",
                Occupied(Black, Pawn) => "♟",
                Occupied(Black, Rook) => "♜",
                Occupied(Black, Knight) => "♞",
                Occupied(Black, Bishop) => "♝",
                Occupied(Black, Queen) => "♛",
                Occupied(Black, King) => "♚",
                Occupied(White, Pawn) => "♙",
                Occupied(White, Rook) => "♖",
                Occupied(White, Knight) => "♘",
                Occupied(White, Bishop) => "♗",
                Occupied(White, Queen) => "♕",
                Occupied(White, King) => "♔",
            }
        )
    }
}

pub const NO: Field = Field::Empty;
pub const BP: Field = Field::Occupied(Colour::Black, Piece::Pawn);
pub const BR: Field = Field::Occupied(Colour::Black, Piece::Rook);
pub const BN: Field = Field::Occupied(Colour::Black, Piece::Knight);
pub const BB: Field = Field::Occupied(Colour::Black, Piece::Bishop);
pub const BQ: Field = Field::Occupied(Colour::Black, Piece::Queen);
pub const BK: Field = Field::Occupied(Colour::Black, Piece::King);
pub const WP: Field = Field::Occupied(Colour::White, Piece::Pawn);
pub const WR: Field = Field::Occupied(Colour::White, Piece::Rook);
pub const WN: Field = Field::Occupied(Colour::White, Piece::Knight);
pub const WB: Field = Field::Occupied(Colour::White, Piece::Bishop);
pub const WQ: Field = Field::Occupied(Colour::White, Piece::Queen);
pub const WK: Field = Field::Occupied(Colour::White, Piece::King);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Board([u8; 32]);

impl Board {
    pub const EMPTY: Self = Self([0; 32]);
    #[inline]
    fn interpret_coords(coords: Coords) -> (usize, bool) {
        let b = coords.into_u8();
        ((b >> 1) as usize, b & 1 == 1)
    }
    #[inline]
    #[track_caller]
    pub fn get(&self, coords: Coords) -> Field {
        let (i, shift_field) = Self::interpret_coords(coords);
        let f = self.0[i];
        if shift_field {
            Field::from_bits(f >> 4)
        } else {
            Field::from_bits(f & 0xf)
        }
    }
    #[inline]
    pub fn set(&mut self, coords: Coords, field: Field) -> Field {
        let get = self.get(coords);
        let (i, shift_field) = Self::interpret_coords(coords);
        if shift_field {
            self.0[i] &= 0x0f;
            self.0[i] |= field.into_bits() << 4;
        } else {
            self.0[i] &= 0xf0;
            self.0[i] |= field.into_bits();
        }
        get
    }
}

pub const START: Board = Board([
    WR.or(WN), WB.or(WQ), WK.or(WB), WN.or(WR),
    WP.or(WP), WP.or(WP), WP.or(WP), WP.or(WP),
    NO.or(NO), NO.or(NO), NO.or(NO), NO.or(NO),
    NO.or(NO), NO.or(NO), NO.or(NO), NO.or(NO),
    NO.or(NO), NO.or(NO), NO.or(NO), NO.or(NO),
    NO.or(NO), NO.or(NO), NO.or(NO), NO.or(NO),
    BP.or(BP), BP.or(BP), BP.or(BP), BP.or(BP),
    BR.or(BN), BB.or(BQ), BK.or(BB), BN.or(BR),
]);

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Board(board) = self;
        writeln!(f, " abcdefgh")?;
        for (i, row) in (0..8).map(|i| 8 - i).zip(board.chunks_exact(4).rev()) {
            write!(f, "{}", i)?;
            for bits in row {
                let (o1, o2) = (Field::from_bits(bits & 0xf), Field::from_bits(bits >> 4));
                write!(f, "{o1}{o2}")?;
            }
            writeln!(f, " {}", i)?;
        }
        writeln!(f, " abcdefgh")
    }
}
