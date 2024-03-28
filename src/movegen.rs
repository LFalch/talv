use std::mem;

use crate::{
    board::{Colour, Field, Piece},
    boardstate::BoardState,
    location::{Coords, Rank, LEAPS},
};

const STRAIGHTS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const CASTLINGS: [(i8, i8); 2] = [(2, 0), (-2, 0)];
const DIAGANOLS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KNIGHTIES: [(i8, i8); 8] = LEAPS;

pub type Move = (Coords, Coords, Option<Piece>);

pub trait AddMove {
    /// Returns an error if it could not add the move due to lack of space
    fn add_move(&mut self, mv: Move) -> Result<(), NoMoreSpace>;
}

#[derive(Debug, Clone, Copy)]
pub struct NoMoreSpace;

pub fn gen_legal_moves<B: AddMove>(buf: &mut B, state: &BoardState) -> Result<(), NoMoreSpace> {
    let mut check_move = |from, unto, promotion| {
        // bit silly
        let mut state = state.clone();
        // Check if move is pseudo-legal and then fully by seeing if it leaves us in check afterwards
        if state.make_move(from, unto, promotion).is_ok() && !state.in_check(!state.side_to_move) {
            buf.add_move((from, unto, promotion))?;
            return Ok(true);
        }
        Ok(false)
    };

    let forwards = match state.side_to_move {
        Colour::Black => -1,
        Colour::White => 1,
    };

    for from in Coords::full_range() {
        match state.board.get(from) {
            Field::Occupied(side, p) if side == state.side_to_move => match p {
                Piece::Pawn => for unto in [
                    (0, 1 * forwards),
                    (0, 2 * forwards),
                    (1, 1 * forwards),
                    (-1, 1 * forwards),
                ]
                .into_iter()
                .filter_map(|(l, n)| from.add(l, n))
                {
                    if unto.r() == Rank::N1 || unto.r() == Rank::N8 {
                        (&mut check_move)(from, unto, Some(Piece::Queen))?;
                        (&mut check_move)(from, unto, Some(Piece::Knight))?;
                        (&mut check_move)(from, unto, Some(Piece::Rook))?;
                        (&mut check_move)(from, unto, Some(Piece::Bishop))?;
                    } else {
                        (&mut check_move)(from, unto, None)?;
                    }
                },
                Piece::Knight => for unto in KNIGHTIES
                    .into_iter()
                    .filter_map(|(l, n)| from.add(l, n))
                    {
                        (&mut check_move)(from, unto, None)?;
                    },
                Piece::King => for unto in STRAIGHTS
                    .into_iter()
                    .chain(DIAGANOLS.into_iter())
                    .chain(CASTLINGS.into_iter())
                    .into_iter()
                    .filter_map(|(l, n)| from.add(l, n))
                    {
                        (&mut check_move)(from, unto, None)?;
                    },
                Piece::Rook => {
                    for (dl, dn) in STRAIGHTS {
                        follow_direction(&mut check_move, from, dl, dn)?;
                    }
                }
                Piece::Bishop => {
                    for (dl, dn) in DIAGANOLS {
                        follow_direction(&mut check_move, from, dl, dn)?;
                    }
                }
                Piece::Queen => {
                    for (dl, dn) in [STRAIGHTS, DIAGANOLS].concat() {
                        follow_direction(&mut check_move, from, dl, dn)?;
                    }
                }
            },
            _ => (),
        }
    }

    Ok(())
}

fn follow_direction<F: FnMut(Coords, Coords, Option<Piece>) -> Result<bool, NoMoreSpace>>(
    check_move: &mut F,
    from: Coords,
    dl: i8,
    dn: i8,
) -> Result<(), NoMoreSpace> {
    for i in 1.. {
        if let Some(unto) = from.add(i * dl, i * dn) {
            if check_move(from, unto, None)? {
                continue;
            }
        }
        break;
    }
    Ok(())
}

#[inline(always)]
pub fn any_legal_moves(state: &BoardState) -> bool {
    gen_legal_moves(&mut (), state).is_err()
}
#[inline(always)]
pub fn get_all_moves(state: &BoardState) -> Vec<Move> {
    let mut vec = Vec::new();
    gen_legal_moves(&mut vec, state).unwrap();
    vec
}

impl AddMove for () {
    #[inline(always)]
    fn add_move(&mut self, _: Move) -> Result<(), NoMoreSpace> {
        Err(NoMoreSpace)
    }
}
impl AddMove for Vec<Move> {
    #[inline(always)]
    fn add_move(&mut self, mv: Move) -> Result<(), NoMoreSpace> {
        self.push(mv);
        Ok(())
    }
}
impl AddMove for &mut [Move] {
    #[inline(always)]
    fn add_move(&mut self, mv: Move) -> Result<(), NoMoreSpace> {
        if self.is_empty() {
            return Err(NoMoreSpace);
        }
        // for some reason this nonsense is necessary
        let (a, b) = mem::take(self).split_at_mut(1);
        a[0] = mv;
        *self = b;
        Ok(())
    }
}
