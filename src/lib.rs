pub mod board;
// pub mod algebraic;
pub mod location;
use std::collections::HashMap;

use board::*;
use location::{Coords, Number, Letter};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct CastlesAllowed {
    short: bool,
    long: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardState {
    board: Board,
    pub side_to_move: Colour,
    black_castling: CastlesAllowed,
    white_castling: CastlesAllowed,
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
    pub fn new() -> Self {
        BoardState {
            board: START,
            side_to_move: Colour::White,
            black_castling: CastlesAllowed {short: true, long: true},
            white_castling: CastlesAllowed {short: true, long: true},
        }
    }
    pub fn in_check(&self, side: Colour) -> bool {
        let king = self.find_king(side);

        for i in 0..8 {
            for j in 0..8 {
                let cs = Coords::from_u8_tuple(i, j).unwrap();

                if self.is_possible(cs, king, !side) {
                    return true;
                }
            }
        }
        false
    }
    fn find_king(&self, c: Colour) -> Coords {
        for i in 0..8 {
            for j in 0..8 {
                let cs = Coords::from_u8_tuple(i, j).unwrap();

                match self.board.get(cs) {
                    Field::Occupied(pc, Piece::King) if pc == c => return cs,
                    _ => (),
                }
            }
        }
        unreachable!();
    }
    pub fn make_move(&mut self, from: Coords, unto: Coords) -> Result<Success, ()> {
        if !self.is_possible(from, unto, self.side_to_move) {
            Err(())
        } else {
            let mover = self.board.set(from, Field::Empty);
            let taken = self.board.set(unto, mover);

            self.update_allowed_castles(mover, from);

            self.side_to_move = !self.side_to_move;

            self.update_allowed_castles(taken, unto);

            let pawn_move = matches!(mover, Field::Occupied(_, Piece::Pawn));
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
    }
    fn update_allowed_castles(&mut self, mover: Field, pos: Coords) {
        let (ac, brn) = match self.side_to_move {
            Colour::Black => (&mut self.black_castling, Number::new(7).unwrap()),
            Colour::White => (&mut self.white_castling, Number::new(0).unwrap()),
        };

        match mover {
            Field::Occupied(_, Piece::King) => {
                ac.short = false;
                ac.long = false;
            }
            Field::Occupied(_, Piece::Rook) if pos.n() == brn => {
                if pos.l() == Letter::new(7).unwrap() {
                    ac.short = false;
                } else if pos.l() == Letter::new(0).unwrap() {
                    ac.long = false;
                }
            }
            _ => (),
        }
    }
    // Determines if the movement is legal except for whether king is in check after
    fn is_possible(&self, from: Coords, unto: Coords, colour_to_move: Colour) -> bool {
        // The two coordinates have to be different
        if from == unto { return false }

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
                let d_num = sign * (unto.n().i8() - from.n().i8());

                if taking {
                    // TODO: en passant
                    from.l() != unto.l() && d_num == 1 && (unto.l().i8() - from.l().i8()).abs() == 1
                } else {
                    if d_num == 1 {
                        true
                    } else if d_num == 2 && 2*from.n().i8() + 5*sign == 7 {
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
            Piece::Queen  => self.check_along(from, unto, |x, y| x == y || x == 0 || y == 0),
            Piece::Rook   => self.check_along(from, unto, |x, y| x == 0 || y == 0),
            Piece::King   => {
                let (dl, dn) = unto.sub(from);
                let (al, an) = (dl.abs(), dn.abs());

                if al <= 1 && an <= 1 {
                    true
                } else if dn == 0 {
                    let ac = match colour_to_move {
                        Colour::Black => self.black_castling,
                        Colour::White => self.white_castling,
                    };
                    !taking &&
                    ((ac.short && dl == 2 && self.board.get(from.add(1, 0).unwrap()).is_empty()) ||
                    (ac.long && dl == -2 && self.board.get(from.add(-1, 0).unwrap()).is_empty()))
                } else {
                    false
                }
            }
        }
    }
    fn check_along<F: FnOnce(i8, i8) -> bool>(&self, from: Coords, unto: Coords, f: F) -> bool {
        let (dl, dn) = unto.sub(from);
        let (al, an) = (dl.abs(), dn.abs());
        if f(al, an) {
            let dl = dl.signum();
            let dn = dn.signum();

            let (l, n) = from.i8_tuple();

            for i in 1..an {
                let coords = Coords::from_u8_tuple(l+i*dl, n+i*dn);

                let is_free = coords
                    .map(|to| self.board.get(to).is_empty());
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
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board_state: BoardState,
    last_move_states: HashMap<BoardState, u8>,
}

impl Game {
    pub fn new() -> Self {
        Game { board_state: BoardState::new(), last_move_states: HashMap::new() }
    }
    pub fn draw_claimable(&self) -> bool {
        self.last_move_states[&self.board_state] == 3
            || self.last_move_states.values().copied().sum::<u8>() == 100
    }
    fn attempt_move(&self, from: Coords, unto: Coords) -> Option<(Success, BoardState)> {
        let mut board_state = self.board_state;

        let success = board_state.make_move(from, unto).ok()?;

        if board_state.in_check(self.board_state.side_to_move) {
            None
        } else {
            Some((success, board_state))
        }
    }
    pub fn make_move(&mut self, from: Coords, unto: Coords) -> bool {
        match self.attempt_move(from, unto) {
            Some((success, new_state)) => {
                self.board_state = new_state;
                match success {
                    Success::PawnMovement | Success::PawnMovementAndCheck | Success::Capture => {
                        self.last_move_states.clear();
                    }
                    Success::Check | Success::PieceMovement => (),
                }
                *self.last_move_states.entry(self.board_state).or_insert(0) += 1;

                true
            }
            None => false,
        }
    }
    pub fn print_board(&self) {
        println!("{}", self.board_state.board);
    }
    pub fn side_to_move(&self) -> Colour {
        self.board_state.side_to_move
    }
    pub fn is_checked(&self, side: Colour) -> bool {
        self.board_state.in_check(side)
    }
}
