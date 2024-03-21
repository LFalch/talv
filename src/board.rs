use std::{fmt::{self, Display}, ops::Not};

use crate::location::Coords;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Piece::*;
        write!(f, "{}", match *self {
            Pawn => "",
            Rook => "R",
            Knight => "N",
            Bishop => "B",
            Queen => "Q",
            King => "K",
        })
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
    pub fn is_empty(&self) -> bool { matches!(*self, Self::Empty) }
    pub fn is_occupied(&self) -> bool { matches!(*self, Self::Occupied(_, _)) }
    pub fn into_piece(self) -> Option<Piece> {
        match self {
            Self::Empty => None,
            Self::Occupied(_, p) => Some(p),
        }
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Piece::*;
        use self::Colour::*;
        use self::Field::*;
        write!(f, "{}", match *self {
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
        })
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
pub struct Board([[Field; 8]; 8]);

impl Board {
    pub fn get(&self, coords: Coords) -> Field {
        let (n, l) = coords.indices();
        self.0[n][l]
    }
    pub fn set(&mut self, coords: Coords, field: Field) -> Field {
        let get = self.get(coords);
        let (n, l) = coords.indices();
        self.0[n][l] = field;
        get
    }
}

pub const START: Board = Board([
    [WR, WN, WB, WQ, WK, WB, WN, WR],
    [WP, WP, WP, WP, WP, WP, BP, WP],
    [NO, NO, NO, NO, NO, NO, NO, NO],
    [NO, NO, NO, NO, NO, NO, NO, NO],
    [NO, NO, NO, NO, NO, NO, NO, NO],
    [NO, NO, NO, NO, NO, NO, NO, NO],
    [BP, BP, BP, BP, BP, BP, BP, BP],
    [BR, BN, BB, BQ, BK, BB, BN, BR],
]);

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Board(board) = self;
        writeln!(f, " abcdefgh")?;
        for (i, row) in (0..8).map(|i| 8 - i).zip(board.iter().rev()) {
            write!(f, "{}", i)?;
            for occupier in row {
                write!(f, "{}", occupier)?;
            }
            writeln!(f, " {}", i)?;
        }
        writeln!(f, " abcdefgh")
    }
}
