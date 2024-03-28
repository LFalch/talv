use std::thread::JoinHandle;

use talv::{board::{Field, Piece}, boardstate::BoardState, bots::bot1, location::{Coords, Rank}};

pub trait Player {
    fn start_interaction(&mut self, _bs: &BoardState, _coords: Coords) { }
    fn get_interaction(&self) -> Option<Piece> { None }
    fn end_interaction(&mut self, _bs: &BoardState, _coords: Coords) { }

    fn make_move(&mut self, bs: &BoardState) -> Option<(Coords, Coords, Option<Piece>)>;
}

#[derive(Debug, Default)]
enum InteractionState {
    #[default]
    NoInteraction,
    Started(Piece, Coords),
    MoveReady(Coords, Coords),
}

use self::InteractionState::*;

#[derive(Debug, Default)]
pub struct HumanPlayer {
    interaction_state: InteractionState,
}

impl Player for HumanPlayer {
    fn start_interaction(&mut self, bs: &BoardState, coords: Coords) {
        match bs.get(coords) {
            Field::Occupied(c, p) if c == bs.side_to_move => {
                self.interaction_state = Started(p, coords);
            }
            _ => (),
        }
    }
    fn get_interaction(&self) -> Option<Piece> {
        match self.interaction_state {
            Started(p, _) => Some(p),
            _ => None,
        }
    }
    fn end_interaction(&mut self, _bs: &BoardState, coords: Coords) {
        match self.interaction_state {
            Started(_, start) => self.interaction_state = MoveReady(start, coords),
            _ => (),
        }
    }

    fn make_move(&mut self, bs: &BoardState) -> Option<(Coords, Coords, Option<Piece>)> {
        match self.interaction_state {
            MoveReady(a, b) => {
                if bs.get(a).into_piece() == Some(Piece::Pawn) && (b.r() == Rank::N1 || b.r() == Rank::N8){
                    // TODO: get a way to specify what to promote to
                    Some((a, b, Some(Piece::Queen)))
                } else {
                    Some((a, b, None))
                }
            },
            _ => None,
        }
    }
}

pub struct Bot1 {
    ongoing: Option<JoinHandle<(f32, Vec<(Coords, Coords, Option<Piece>)>)>>,
}
impl Bot1 {
    pub fn new() -> Self {
        Self {
            ongoing: None,
        }
    }
}
impl Player for Bot1 {
    fn make_move(&mut self, bs: &BoardState) -> Option<(Coords, Coords, Option<Piece>)> {
        let Some(ongoing) = self.ongoing.take() else {
            let bs = bs.clone();
            self.ongoing = Some(std::thread::spawn(move || {
                bot1::get_moves_ranked(&bs, 10, 1_000_000)
            }));
            return None;
        };

        if ongoing.is_finished() {
            let (eval, moves) = ongoing.join().unwrap();

            let (f, t, p) = moves[0];
            println!("{eval}");
            Some((f, t, p))
        } else {
            self.ongoing = Some(ongoing);
            None
        }
    }
}
