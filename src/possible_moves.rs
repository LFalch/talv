use crate::{
    board::{Colour, Field, Piece},
    boardstate::BoardState,
    location::{Coords, Rank, LEAPS},
};

const STRAIGHTS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const CASTLINGS: [(i8, i8); 2] = [(2, 0), (-2, 0)];
const DIAGANOLS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KNIGHTIES: [(i8, i8); 8] = LEAPS;

pub fn possible_moves(state: &BoardState) -> Vec<(Piece, Coords, Coords, Option<Piece>)> {
    let mut possible_moves = Vec::new();

    let mut check_move = |p, from, unto, promotion| {
        // bit silly
        let mut state = state.clone();
        // Check if move is pseudo-legal and then fully by seeing if it leaves us in check afterwards
        if state.make_move(from, unto, promotion).is_ok() && !state.in_check(!state.side_to_move) {
            possible_moves.push((p, from, unto, promotion));
            return true;
        }
        false
    };

    let forwards = match state.side_to_move {
        Colour::Black => -1,
        Colour::White => 1,
    };

    for from in Coords::full_range() {
        match state.board.get(from) {
            Field::Occupied(side, p) if side == state.side_to_move => match p {
                Piece::Pawn => [
                    (0, 1 * forwards),
                    (0, 2 * forwards),
                    (1, 1 * forwards),
                    (-1, 1 * forwards),
                ]
                .into_iter()
                .filter_map(|(l, n)| from.add(l, n))
                .for_each(|unto| {
                    if unto.r() == Rank::N1 || unto.r() == Rank::N8 {
                        (&mut check_move)(p, from, unto, Some(Piece::Queen));
                        (&mut check_move)(p, from, unto, Some(Piece::Knight));
                        (&mut check_move)(p, from, unto, Some(Piece::Rook));
                        (&mut check_move)(p, from, unto, Some(Piece::Bishop));
                    } else {
                        (&mut check_move)(p, from, unto, None);
                    }
                }),
                Piece::Knight => KNIGHTIES
                    .into_iter()
                    .filter_map(|(l, n)| from.add(l, n))
                    .for_each(|unto| {
                        (&mut check_move)(p, from, unto, None);
                    }),
                Piece::King => STRAIGHTS
                    .into_iter()
                    .chain(DIAGANOLS.into_iter())
                    .chain(CASTLINGS.into_iter())
                    .into_iter()
                    .filter_map(|(l, n)| from.add(l, n))
                    .for_each(|unto| {
                        (&mut check_move)(p, from, unto, None);
                    }),
                Piece::Rook => {
                    for (dl, dn) in STRAIGHTS {
                        follow_direction(&mut check_move, from, p, dl, dn);
                    }
                }
                Piece::Bishop => {
                    for (dl, dn) in DIAGANOLS {
                        follow_direction(&mut check_move, from, p, dl, dn);
                    }
                }
                Piece::Queen => {
                    for (dl, dn) in [STRAIGHTS, DIAGANOLS].concat() {
                        follow_direction(&mut check_move, from, p, dl, dn);
                    }
                }
            },
            _ => (),
        }
    }

    possible_moves
}

fn follow_direction<F: FnMut(Piece, Coords, Coords, Option<Piece>) -> bool>(
    check_move: &mut F,
    from: Coords,
    p: Piece,
    dl: i8,
    dn: i8,
) {
    for i in 1.. {
        if let Some(unto) = from.add(i * dl, i * dn) {
            if check_move(p, from, unto, None) {
                continue;
            }
        }
        break;
    }
}
