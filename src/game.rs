use std::{
    collections::HashMap,
    fmt::{self, Display},
    num::NonZeroU64,
};

use crate::boardstate::{BoardState, Success};

use super::algebraic::{Move, MoveType, Mover};
use super::board::*;
use super::location::{Coords, File, FileRange, Rank, RankRange};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board_state: BoardState,
    last_move_states: HashMap<BoardState, u8>,
    fullmove_count: NonZeroU64,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board_state: BoardState::new(),
            last_move_states: HashMap::new(),
            fullmove_count: NonZeroU64::new(1).unwrap(),
        }
    }
    pub fn from_fen(fen: &str) -> Option<Self> {
        let move_count_index = fen.rfind(char::is_whitespace)?;
        let fullmove_count = fen[move_count_index..].trim_start().parse().ok()?;
        let half_move_clock_index = fen[..move_count_index].rfind(char::is_whitespace)?;

        let mut last_move_states = HashMap::new();
        // Set an impossible board state that will contribute to the fifty-move rule
        last_move_states.insert(
            BoardState {
                board: Board::EMPTY,
                ..BoardState::new()
            },
            fen[half_move_clock_index..move_count_index]
                .trim_start()
                .parse()
                .ok()?,
        );

        let board_state = BoardState::from_fen(&fen[..half_move_clock_index])?;

        Some(Game {
            board_state,
            last_move_states,
            fullmove_count,
        })
    }
    pub fn draw_claimable(&self) -> bool {
        self.last_move_states.get(&self.board_state).copied().unwrap_or(0) == 3
        || self.last_move_states.values().copied().sum::<u8>() == 100 || 'only_kings: {
            // Check if only kings are left
            for cs in Coords::full_range() {
                match self.board_state.get(cs) {
                    Field::Occupied(_, Piece::King) | Field::Empty => (),
                    _ => break 'only_kings false,
                }
            }
            true
        }
    }
    fn attempt_move(&self, from: Coords, unto: Coords, promotion: Option<Piece>) -> Option<(Success, BoardState)> {
        let mut board_state = self.board_state;

        let success = board_state.make_move(from, unto, promotion).ok()?;

        if board_state.in_check(self.board_state.side_to_move) {
            None
        } else {
            Some((success, board_state))
        }
    }
    pub fn make_move(&mut self, from: Coords, unto: Coords, promotion: Option<Piece>) -> bool {
        match self.attempt_move(from, unto, promotion) {
            Some((success, new_state)) => {
                self.board_state = new_state;
                match success {
                    Success::PawnMovement | Success::PawnMovementAndCheck | Success::Capture => {
                        self.last_move_states.clear();
                    }
                    Success::Check | Success::PieceMovement => (),
                }
                *self.last_move_states.entry(self.board_state).or_insert(0) += 1;
                if matches!(self.side_to_move(), Colour::White) {
                    self.fullmove_count = self.fullmove_count.checked_add(1).unwrap();
                }

                true
            }
            None => false,
        }
    }
    pub fn print_game(&self) {
        println!(
            "Move {}, {} to move",
            self.fullmove_count,
            match self.side_to_move() {
                Colour::White => "white",
                Colour::Black => "black",
            }
        );
        println!("{}", self.board_state.board);
    }
    pub fn board_state(&self) -> &BoardState {
        &self.board_state
    }
    pub fn side_to_move(&self) -> Colour {
        self.board_state.side_to_move
    }
    pub fn is_checked(&self, side: Colour) -> bool {
        self.board_state.in_check(side)
    }
    // Ignores check and checkmates
    pub fn check_move(&self, alg_move: Move) -> Option<(Coords, Coords, Option<Piece>)> {
        let to_play = self.board_state.side_to_move;

        let (ca, brn) = match self.board_state.side_to_move {
            Colour::Black => (self.board_state.black_castling, Rank::N8),
            Colour::White => (self.board_state.white_castling, Rank::N1),
        };

        let capturing = |destination| {
            self.board_state.board.get(destination).is_occupied()
                || self.board_state.en_passant_target == Some(destination)
        };

        Some(match alg_move.move_type {
            MoveType::ShortCastle if ca.short => {
                (Coords::new(File::E, brn), Coords::new(File::G, brn), None)
            }
            MoveType::LongCastle if ca.long => {
                (Coords::new(File::E, brn), Coords::new(File::C, brn), None)
            }
            MoveType::Regular {
                captures,
                destination,
                ..
            } if captures != capturing(destination) => return None,
            MoveType::Regular {
                mover,
                destination: unto,
                promotes,
                ..
            } => {
                // If a move is a pawn going to a back rank, it should be a promotion move
                if mover.is_pawn()
                    && (unto.r() == Rank::N8 || unto.r() == Rank::N1)
                    && promotes.is_none()
                {
                    return None;
                }

                (
                    match mover {
                        Mover::PieceAt(p, from) => {
                            match self.board_state.board.get(from) {
                                // Pawn is implied, but if we have `pos -> pos`, then it's a wildcard
                                Field::Occupied(c, p2)
                                    if c == to_play && p == Piece::Pawn || p == p2 =>
                                {
                                    from
                                }
                                _ => return None,
                            }
                        }
                        Mover::PieceAtLetter(p, l) => {
                            let mut move_from = None;
                            for n in RankRange::full() {
                                let coords = Coords::new(l, n);
                                match self.board_state.board.get(coords) {
                                    Field::Occupied(c, p2)
                                        if c == to_play
                                            && p2 == p
                                            && self.attempt_move(coords, unto, promotes).is_some() =>
                                    {
                                        if move_from.is_some() {
                                            // Ambiguous
                                            return None;
                                        } else {
                                            move_from = Some(coords);
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            move_from?
                        }
                        Mover::PieceAtNumber(p, n) => {
                            let mut move_from = None;
                            for l in FileRange::full() {
                                let coords = Coords::new(l, n);
                                match self.board_state.board.get(coords) {
                                    Field::Occupied(c, p2)
                                        if c == to_play
                                            && p2 == p
                                            && self.attempt_move(coords, unto, promotes).is_some() =>
                                    {
                                        if move_from.is_some() {
                                            // Ambiguous
                                            return None;
                                        } else {
                                            move_from = Some(coords);
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            move_from?
                        }
                        Mover::Piece(p) => {
                            let mut move_from = None;
                            for n in RankRange::full() {
                                for l in FileRange::full() {
                                    let coords = Coords::new(l, n);
                                    match self.board_state.board.get(coords) {
                                        Field::Occupied(c, p2)
                                            if c == to_play
                                                && p2 == p
                                                && self.attempt_move(coords, unto, promotes).is_some() =>
                                        {
                                            if move_from.is_some() {
                                                // Ambiguous
                                                return None;
                                            } else {
                                                move_from = Some(coords);
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            move_from?
                        }
                    },
                    unto,
                    promotes
                )
            }
            _ => return None,
        })
    }
    pub const fn display_fen(&self) -> GameFen {
        GameFen { inner: self }
    }
}

pub struct GameFen<'a> {
    inner: &'a Game,
}

impl Display for GameFen<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Game {
            board_state,
            last_move_states,
            fullmove_count,
        } = &self.inner;
        write!(
            f,
            "{} {} {fullmove_count}",
            board_state.display_fen(),
            last_move_states.values().sum::<u8>()
        )
    }
}
