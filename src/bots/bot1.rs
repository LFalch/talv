use std::collections::HashMap;

use crate::{board::{Field, Piece}, boardstate::BoardState, location::Coords, possible_moves::possible_moves};

pub type Move = (Coords, Coords);

fn search(state: &BoardState, alpha: f32, beta: f32, depth: usize, previous: &mut HashMap<BoardState, f32>) -> f32 {
    if let Some(v) = previous.get(state).copied() {
        return v;
    }

    let v = search_inner(state, alpha, beta, depth, previous);
    previous.insert(state.clone(), v);
    v
}
fn search_inner(state: &BoardState, mut alpha: f32, beta: f32, depth: usize, previous: &mut HashMap<BoardState, f32>) -> f32 {
    if depth == 0 {
        return eval(state);
    }

    let possible_moves = possible_moves(state);

    if possible_moves.is_empty() {
        return eval(state);
    }

    let mut best_eval = f32::NEG_INFINITY;

    for (_, f, t) in possible_moves {
        let mut new_state = state.clone();
        new_state.make_move(f, t).unwrap();

        let eval = -search(&new_state, beta, alpha, depth-1, previous);

        if eval > best_eval {
            best_eval = eval;
            alpha = if alpha.is_nan() { eval } else { alpha.max(eval) };
            if beta <= alpha {
                break;
            }
        }
    }

    best_eval
}

pub fn get_moves_ranked(state: &BoardState) -> Vec<(f32, Coords, Coords)> {
    const INITIAL_DEPTH: usize = 4;
    let possible_moves = possible_moves(state);

    let mut moves_with_eval = Vec::with_capacity(possible_moves.len());
    {
        let mut previous = HashMap::with_capacity(1024);

        'evaluate_possible_moves: for (_, from, unto) in possible_moves {
            let mut new_state = state.clone();
            new_state.make_move(from, unto).unwrap();
            let eval = -search(&new_state, f32::NAN, f32::NAN, INITIAL_DEPTH, &mut previous);

            for (i, &(e, _, _)) in moves_with_eval.iter().enumerate() {
                if eval > e {
                    moves_with_eval.insert(i, (eval, from, unto));
                    continue 'evaluate_possible_moves;
                }
            }
            moves_with_eval.push((eval, from, unto))
        }
    }

    moves_with_eval
}

/// Positive value => good for current last player
fn eval(state: &BoardState) -> f32 {
    if possible_moves(state).is_empty() {
        if state.in_check(state.side_to_move) {
            // I'm in a checkmate!!! oh no!
            return f32::NEG_INFINITY;
        } else {
            // draw :/
            return 0.;
        }
    }
    let mut checking_bonus = 0.;
    if state.in_check(!state.side_to_move) {
        checking_bonus += 10.;
        let mut new_state = state.clone();
        new_state.side_to_move = !new_state.side_to_move;
        if possible_moves(&new_state).is_empty() {
            return f32::INFINITY;
        }
    }

    eval_pieces(state) + checking_bonus
}
fn eval_pieces(state: &BoardState) -> f32 {
    let mut piece_difference = 0.;
    let mut abs_piece_sum = 0.;
    for field in Coords::full_range().map(|c| state.board.get(c)) {
        match field {
            Field::Empty => (),
            Field::Occupied(c, p) => {
                let value = piece_value(p);
                abs_piece_sum += value;
                if c == state.side_to_move {
                    piece_difference += value;
                } else {
                    piece_difference -= value;
                }
            }
        }
    }
    piece_difference / abs_piece_sum
}

const fn piece_value(piece: Piece) -> f32 {
    match piece {
        Piece::Pawn => 1.,
        Piece::Knight => 3.,
        Piece::Bishop => 3.,
        Piece::Rook => 5.,
        Piece::Queen => 9.,
        // cannot use infinity for this as it would make the average useless
        Piece::King => 0.,
    }
}
