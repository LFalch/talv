use crate::{
    board::{Colour, Field, Piece},
    boardstate::BoardState,
    location::{Coords, FileRange, RankRange, LEAPS},
};

const STRAIGHTS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const CASTLINGS: [(i8, i8); 2] = [(2, 0), (-2, 0)];
const DIAGANOLS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const KNIGHTIES: [(i8, i8); 8] = LEAPS;

pub fn possible_moves(state: &BoardState) -> Vec<(Piece, Coords, Coords)> {
    let mut pieces = Vec::new();
    for n in RankRange::full() {
        for l in FileRange::full() {
            let from = Coords::new(l, n);
            match state.board.get(from) {
                Field::Occupied(side, p) if side == state.side_to_move => pieces.push((from, p)),
                _ => (),
            }
        }
    }

    let mut possible_moves = Vec::new();

    let mut check_move = |p, from, unto| {
        // bit silly
        let mut state = state.clone();
        // Check if move is pseudo-legal and then fully by seeing if it leaves us in check afterwards
        if state.make_move(from, unto).is_ok() && !state.in_check(!state.side_to_move) {
            possible_moves.push((p, from, unto));
            return true;
        }
        false
    };

    let forwards = match state.side_to_move {
        Colour::Black => -1,
        Colour::White => 1,
    };

    for (from, p) in pieces {
        match p {
            Piece::Pawn => [
                (0, 1 * forwards),
                (0, 2 * forwards),
                (1, 1 * forwards),
                (-1, 1 * forwards),
            ]
            .into_iter()
            .filter_map(|(l, n)| from.add(l, n))
            .for_each(|unto| {
                (&mut check_move)(p, from, unto);
            }),
            Piece::Knight => KNIGHTIES
                .into_iter()
                .filter_map(|(l, n)| from.add(l, n))
                .for_each(|unto| {
                    (&mut check_move)(p, from, unto);
                }),
            Piece::King => STRAIGHTS.into_iter()
                .chain(DIAGANOLS.into_iter())
                .chain(CASTLINGS.into_iter())
                .into_iter()
                .filter_map(|(l, n)| from.add(l, n))
                .for_each(|unto| {
                    (&mut check_move)(p, from, unto);
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
        }
    }

    possible_moves
}

fn follow_direction<F: FnMut(Piece, Coords, Coords) -> bool>(
    check_move: &mut F,
    from: Coords,
    p: Piece,
    dl: i8,
    dn: i8,
) {
    for i in 1.. {
        if let Some(unto) = from.add(i * dl, i * dn) {
            if check_move(p, from, unto) {
                continue;
            }
        }
        break;
    }
}
