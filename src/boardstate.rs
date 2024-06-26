use std::fmt::{self, Display};

use super::board::*;
use super::location::{Coords, File, FileRange, Rank, RankRange};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CastlesAllowed {
    pub(crate) short: bool,
    pub(crate) long: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardState {
    pub(crate) board: Board,
    pub side_to_move: Colour,
    pub(crate) black_castling: CastlesAllowed,
    pub(crate) white_castling: CastlesAllowed,
    pub(crate) en_passant_target: Option<Coords>,
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState::new()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Success {
    Capture,
    PawnMovement,
    PawnMovementAndCheck,
    Check,
    PieceMovement,
}

impl BoardState {
    pub const fn new() -> Self {
        BoardState {
            board: START,
            side_to_move: Colour::White,
            black_castling: CastlesAllowed {
                short: true,
                long: true,
            },
            white_castling: CastlesAllowed {
                short: true,
                long: true,
            },
            en_passant_target: None,
        }
    }
    /// Reads a board state from the first four fields of a FEN string
    pub fn from_fen(s: &str) -> Option<Self> {
        let mut fields = s.split_whitespace();

        let mut board = Board::EMPTY;

        let pieces = fields.next()?;

        let mut ns = RankRange::full().rev();
        let mut n = ns.next().unwrap();
        let mut ls = FileRange::full();
        for c in pieces.chars() {
            match c {
                '/' => {
                    if ls.next().is_some() {
                        // assert this is the last letter
                        return None;
                    }
                    n = ns.next()?;
                    ls = FileRange::full();
                }
                c @ '1'..='8' => {
                    for _ in '0'..c {
                        ls.next()?;
                    }
                }
                'p' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::Black, Piece::Pawn),
                    );
                }
                'r' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::Black, Piece::Rook),
                    );
                }
                'n' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::Black, Piece::Knight),
                    );
                }
                'b' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::Black, Piece::Bishop),
                    );
                }
                'q' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::Black, Piece::Queen),
                    );
                }
                'k' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::Black, Piece::King),
                    );
                }
                'P' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::White, Piece::Pawn),
                    );
                }
                'R' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::White, Piece::Rook),
                    );
                }
                'N' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::White, Piece::Knight),
                    );
                }
                'B' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::White, Piece::Bishop),
                    );
                }
                'Q' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::White, Piece::Queen),
                    );
                }
                'K' => {
                    board.set(
                        Coords::new(ls.next()?, n),
                        Field::Occupied(Colour::White, Piece::King),
                    );
                }
                _ => return None,
            }
        }

        let side_to_move = match fields.next()? {
            "w" => Colour::White,
            "b" => Colour::Black,
            _ => return None,
        };

        let mut black_castling = CastlesAllowed {
            short: false,
            long: false,
        };
        let mut white_castling = CastlesAllowed {
            short: false,
            long: false,
        };

        for c in fields.next()?.chars() {
            match c {
                '-' => break,
                'K' => white_castling.short = true,
                'Q' => white_castling.long = true,
                'k' => black_castling.short = true,
                'q' => black_castling.long = true,
                _ => return None,
            }
        }

        let en_passant_target = match fields.next()? {
            "-" => None,
            s => Some(Coords::from_str(s)?),
        };

        Some(BoardState {
            board,
            side_to_move,
            black_castling,
            white_castling,
            en_passant_target,
        })
    }
    pub fn in_check(&self, side: Colour) -> bool {
        let king = self.find_king(side);

        self.is_threatened(king, !side)
    }
    fn is_threatened(&self, spot: Coords, by_side: Colour) -> bool {
        for cs in Coords::full_range() {
            if self.is_pseudo_legal(by_side, cs, spot) {
                return true;
            }
        }
        false
    }
    fn find_king(&self, c: Colour) -> Coords {
        for cs in Coords::full_range() {
            match self.board.get(cs) {
                Field::Occupied(pc, Piece::King) if pc == c => return cs,
                _ => (),
            }
        }
        unreachable!("no king");
    }
    pub fn make_move(&mut self, from: Coords, unto: Coords, promotion: Option<Piece>) -> Result<Success, ()> {
        if !self.is_pseudo_legal(self.side_to_move, from, unto) {
            return Err(())
        }
        // Check promotion
        let legal_promotion;
        if self.board.get(from).into_piece() == Some(Piece::Pawn) {
            legal_promotion = match promotion {
                None => unto.r() != Rank::N1 && unto.r() != Rank::N8,
                Some(Piece::King | Piece::Pawn) => false,
                Some(_) => unto.r() == Rank::N1 || unto.r() == Rank::N8,
            };
        } else {
            legal_promotion = promotion.is_none();
        }
        if !legal_promotion {
            return Err(())
        }
        // Check castling
        let dist = unto.sub(from);
        if dist.0.abs() == 2 && self.board.get(from).into_piece() == Some(Piece::King) {
            if self.in_check(self.side_to_move) || self.is_threatened(from.add(dist.0/2, 0).unwrap(), !self.side_to_move) {
                return Err(());
            }
        }

        let mover = self.board.set(from, Field::Empty);
        let taken = match self.en_passant_target {
            Some(en_passant_target) if unto == en_passant_target && matches!(mover, Field::Occupied(_, Piece::Pawn)) => {
                let targeted_pawn_pos = match en_passant_target.r() {
                    // FIXME: probably do this better
                    Rank::N3 => en_passant_target.add(0, 1).unwrap(),
                    Rank::N6 => en_passant_target.add(0, -1).unwrap(),
                    _ => unreachable!(),
                };

                // this should be empty because otherwise the board was in an illegal state
                let _ = self.board.set(unto, mover);
                // Kill the pawn
                self.board.set(targeted_pawn_pos, Field::Empty)
            }
            // if this is not en passant capture, this is straight forward
            _ => if let Some(new_piece) = promotion {
                let mover = match mover {
                    Field::Occupied(c, _) => Field::Occupied(c, new_piece),
                    _ => unreachable!(),
                };
                self.board.set(unto, mover)
            } else {
                self.board.set(unto, mover)
            },
        };

        self.update_allowed_castles(mover, from);

        self.side_to_move = !self.side_to_move;

        self.update_allowed_castles(taken, unto);

        let pawn_move = matches!(mover, Field::Occupied(_, Piece::Pawn));

        if pawn_move && dist.1.abs() == 2 {
            // Set up en passant
            let target_pos = unto.add(0, -dist.1 / 2).unwrap();
            self.en_passant_target = Some(target_pos);
        } else {
            self.en_passant_target = None;
            // Castling
            if matches!(mover, Field::Occupied(_, Piece::King)) && dist.0.abs() == 2 {
                // FIXME: not pretty
                match dist.0.signum() {
                    1 => {
                        let rook = self
                            .board
                            .set(Coords::new(File::H, unto.r()), Field::Empty);
                        self.board.set(unto.add(-1, 0).unwrap(), rook);
                    }
                    -1 => {
                        let rook = self
                            .board
                            .set(Coords::new(File::A, unto.r()), Field::Empty);
                        self.board.set(unto.add(1, 0).unwrap(), rook);
                    }
                    _ => unreachable!(),
                }
            }
        }

        let check = self.in_check(self.side_to_move);

        if taken.is_occupied() {
            Ok(Success::Capture)
        } else {
            Ok(match (pawn_move, check) {
                (true, true) => Success::PawnMovementAndCheck,
                (true, false) => Success::PawnMovement,
                (false, true) => Success::Check,
                (false, false) => Success::PieceMovement,
            })
        }
    }
    fn update_allowed_castles(&mut self, mover: Field, pos: Coords) {
        let (ac, brn) = match self.side_to_move {
            Colour::Black => (&mut self.black_castling, Rank::N8),
            Colour::White => (&mut self.white_castling, Rank::N1),
        };

        match mover {
            Field::Occupied(_, Piece::King) => {
                ac.short = false;
                ac.long = false;
            }
            Field::Occupied(_, Piece::Rook) if pos.r() == brn => {
                if pos.f() == File::H {
                    ac.short = false;
                } else if pos.f() == File::A {
                    ac.long = false;
                }
            }
            _ => (),
        }
    }
    /// Determines if the movement is legal except for whether king is in check after
    pub fn is_pseudo_legal(&self, colour_to_move: Colour, from: Coords, unto: Coords) -> bool {
        // The two coordinates have to be different
        if from == unto {
            return false;
        }

        let mover = match self.board.get(from) {
            Field::Empty => return false,
            Field::Occupied(c, _) if colour_to_move != c => return false,
            Field::Occupied(_, p) => p,
        };
        let taking = match self.board.get(unto) {
            Field::Occupied(c, _) if c == colour_to_move => return false,
            Field::Empty => false,
            Field::Occupied(_, _) => true,
        };

        match mover {
            Piece::Pawn => {
                let sign = match colour_to_move {
                    Colour::Black => -1,
                    Colour::White => 1,
                };
                let d_num = sign * (unto.r().i8() - from.r().i8());

                // Handle en passant
                let taking = taking || self.en_passant_target == Some(unto);

                // same file <=> !taking
                if (from.f() != unto.f()) != taking {
                    return false;
                }

                if taking {
                    d_num == 1 && (unto.f().i8() - from.f().i8()).abs() == 1
                } else {
                    if d_num == 1 {
                        true
                    } else if d_num == 2 && 2 * from.r().i8() + 5 * sign == 7 {
                        self.board.get(from.add(0, sign).unwrap()).is_empty()
                    } else {
                        false
                    }
                }
            }
            Piece::Knight => {
                let (l, n) = unto.sub(from);
                let (l, n) = (l.abs(), n.abs());

                (l == 2 && n == 1) || (l == 1 && n == 2)
            }
            Piece::Bishop => self.check_along(from, unto, |x, y| x == y),
            Piece::Queen => self.check_along(from, unto, |x, y| x == y || x == 0 || y == 0),
            Piece::Rook => self.check_along(from, unto, |x, y| x == 0 || y == 0),
            Piece::King => {
                let (dl, dn) = unto.sub(from);
                let (al, an) = (dl.abs(), dn.abs());

                if al <= 1 && an <= 1 {
                    true
                } else if dn == 0 {
                    let ac = match colour_to_move {
                        Colour::Black => self.black_castling,
                        Colour::White => self.white_castling,
                    };
                    !taking
                        && ((ac.short
                            && dl == 2
                            && self.board.get(from.add(1, 0).unwrap()).is_empty())
                            || (ac.long
                                && dl == -2
                                && self.board.get(from.add(-1, 0).unwrap()).is_empty()))
                } else {
                    false
                }
            }
        }
    }
    fn check_along<F: FnOnce(i8, i8) -> bool>(&self, from: Coords, unto: Coords, f: F) -> bool {
        let (dl, dn) = unto.sub(from);
        let (al, an) = (dl.abs(), dn.abs());
        let distance = al.max(an);

        if f(al, an) {
            let dl = dl.signum();
            let dn = dn.signum();

            let (l, n) = from.i8_tuple();

            for i in 1..distance {
                let coords = Coords::from_u8_tuple(l + i * dl, n + i * dn);

                let is_free = coords.map(|to| self.board.get(to).is_empty());
                match is_free {
                    Some(true) => (),
                    _ => return false,
                }
            }
            true
        } else {
            false
        }
    }
    pub const fn display_fen(&self) -> BoardStateFen {
        BoardStateFen { inner: self }
    }
    pub fn get(&self, coords: Coords) -> Field {
        self.board.get(coords)
    }
}

pub struct BoardStateFen<'a> {
    inner: &'a BoardState,
}

impl Display for BoardStateFen<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for n in RankRange::full().rev() {
            let mut empty_fields = 0;
            for l in FileRange::full() {
                let field = self.inner.board.get(Coords::new(l, n));
                let Field::Occupied(c, p) = field else {
                    empty_fields += 1;
                    continue;
                };
                if empty_fields > 0 {
                    write!(f, "{empty_fields}")?;
                    empty_fields = 0;
                }
                match (c, p) {
                    (Colour::White, Piece::Pawn) => write!(f, "P")?,
                    (Colour::Black, Piece::Pawn) => write!(f, "p")?,
                    (Colour::White, Piece::Rook) => write!(f, "R")?,
                    (Colour::Black, Piece::Rook) => write!(f, "r")?,
                    (Colour::White, Piece::Knight) => write!(f, "N")?,
                    (Colour::Black, Piece::Knight) => write!(f, "n")?,
                    (Colour::White, Piece::Bishop) => write!(f, "B")?,
                    (Colour::Black, Piece::Bishop) => write!(f, "b")?,
                    (Colour::White, Piece::Queen) => write!(f, "Q")?,
                    (Colour::Black, Piece::Queen) => write!(f, "q")?,
                    (Colour::White, Piece::King) => write!(f, "K")?,
                    (Colour::Black, Piece::King) => write!(f, "k")?,
                }
            }
            if empty_fields > 0 {
                write!(f, "{empty_fields}")?;
            }
            if n != Rank::N1 {
                write!(f, "/")?;
            }
        }

        match self.inner.side_to_move {
            Colour::Black => write!(f, " b ")?,
            Colour::White => write!(f, " w ")?,
        }

        let mut no_castling = true;
        let iter = [
            self.inner.white_castling.short,
            self.inner.white_castling.long,
            self.inner.black_castling.short,
            self.inner.black_castling.long,
        ]
        .into_iter()
        .zip("KQkq".chars())
        .filter_map(|(b, c)| b.then_some(c));
        for c in iter {
            no_castling = false;
            write!(f, "{c}")?;
        }
        if no_castling {
            write!(f, "-")?;
        }

        if let Some(en_passant_target) = self.inner.en_passant_target {
            write!(f, " {en_passant_target}")
        } else {
            write!(f, " -")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_position_fen() {
        let start_from_fen =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        assert_eq!(start_from_fen, BoardState::new());
    }
}
