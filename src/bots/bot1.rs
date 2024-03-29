use std::{collections::HashMap, convert::identity};

use crate::{board::{Colour, Field, Piece}, boardstate::BoardState, location::{Coords, File, Rank}, movegen::{any_legal_moves, gen_legal_moves, get_all_moves}};

pub type Move = (Coords, Coords, Option<Piece>);
const NULL_MOVE: Move = (Coords::new(File::A, Rank::N1), Coords::new(File::A, Rank::N1), None);

type Transpositions = HashMap<BoardState, (usize, f32)>;

struct SearchResult {
    ordered_moves: Vec<Move>,
    nodes: usize,
    eval: f32,
}

fn start_search(state: &BoardState, moves: &[Move], depth: usize, transpositions: &mut Transpositions, max_nodes: usize) -> SearchResult {
    assert_ne!(depth, 0);

    let mut evals = Vec::with_capacity(moves.len());
    let mut ordered_moves = Vec::with_capacity(moves.len());
    for &(f, t, prm) in moves {
        let mut new_state = state.clone();
        new_state.make_move(f, t, prm).unwrap();

        let beta = evals.get(0).copied().unwrap_or(f32::NAN);
        let eval = -search(&new_state, f32::NAN, -beta, depth-1, transpositions, max_nodes);

        let i = evals.binary_search_by(|e| eval.total_cmp(e)).unwrap_or_else(identity);
        evals.insert(i, eval);
        ordered_moves.insert(i, (f, t, prm));
    }

    SearchResult {
        nodes: transpositions.len(),
        ordered_moves,
        eval: evals.get(0).copied().unwrap_or(0.),
    }
}
fn search(state: &BoardState, alpha: f32, beta: f32, depth: usize, transpositions: &mut Transpositions, max_nodes: usize) -> f32 {
    if let Some((d, v)) = transpositions.get(state).copied() {
        if d >= depth {
            return v;
        }
    }

    let v = search_inner(state, alpha, beta, depth, transpositions, max_nodes);
    transpositions.insert(state.clone(), (depth, v));
    v
}
fn search_inner(state: &BoardState, mut alpha: f32, beta: f32, depth: usize, transpositions: &mut Transpositions, max_nodes: usize) -> f32 {
    if depth == 0 || transpositions.len() >= max_nodes {
        let evaluation;
        if let Some((_, v)) = transpositions.get(state).copied() {
            evaluation = v
        } else {
            evaluation = eval(state);
        }
        return evaluation;
    }

    let mut buf;
    let possible_moves = {
        const MAX_MOVES: usize = 200;
        buf = [NULL_MOVE; MAX_MOVES];
        let mut slice = &mut buf[..];

        gen_legal_moves(&mut slice, state).expect("max moves exceeded");
        let unused = slice.len(); 
        &buf[..MAX_MOVES - unused]
    };

    if possible_moves.is_empty() {
        return eval(state);
    }

    for &(f, t, prm) in possible_moves {
        let mut new_state = state.clone();
        new_state.make_move(f, t, prm).unwrap();

        let eval = -search(&new_state, -beta, -alpha, depth-1, transpositions, max_nodes);

        if alpha.is_nan() || eval > alpha {
            // This will give `eval` if alpha is nan
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
    }

    alpha
}

pub fn get_moves_ranked(state: &BoardState, max_depth: usize, max_nodes: usize) -> (f32, Vec<Move>) {
    let possible_moves = get_all_moves(state);

    let mut eval = f32::NAN;
    let mut moves = possible_moves;

    let mut transpositions = Transpositions::with_capacity(1024);

    for depth in 1..=max_depth {
        let res = start_search(state, &moves, depth, &mut transpositions, max_nodes);

        moves = res.ordered_moves;
        eval = res.eval;
        if res.nodes > max_nodes {
            break;
        }
    }

    (eval, moves)
}

/// Positive value => good for current last player
fn eval(state: &BoardState) -> f32 {
    if !any_legal_moves(state) {
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
        if !any_legal_moves(&new_state) {
            return f32::INFINITY;
        }
    }

    eval_pieces(state) + checking_bonus
}
fn eval_pieces(state: &BoardState) -> f32 {
    let mut piece_difference = 0.;
    let mut piece_total = 0.;
    for cs in Coords::full_range() {
        match state.board.get(cs) {
            Field::Empty => (),
            Field::Occupied(c, p) => {
                piece_total += 1.;

                let (f, r) = cs.i8_tuple();
                let r = match c {
                    Colour::White => r,
                    Colour::Black => 7 - r,
                };

                let value = piece_value(f, r, p);
                if c == state.side_to_move {
                    piece_difference += value;
                } else {
                    piece_difference -= value;
                }
            }
        }
    }
    piece_difference / piece_total
}

fn piece_value(f: i8, r: i8, piece: Piece) -> f32 {
    let _ = f;
    match piece {
        Piece::Pawn => 1. + 0.1 * (r as f32).powf(1.1),
        Piece::Knight => 3.,
        Piece::Bishop => 3.2,
        Piece::Rook => 5.,
        Piece::Queen => 9.,
        // cannot use infinity for this as it would make the average useless
        Piece::King => 0.,
    }
}
