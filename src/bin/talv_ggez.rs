use std::{env, path::PathBuf};

use ggez::{
    conf::{WindowMode, WindowSetup}, event::{EventHandler, MouseButton}, graphics::{self, Canvas, Color, DrawMode, DrawParam, Image, Mesh, Rect}, Context, ContextBuilder, GameError
};
use player::{Bot1, HumanPlayer, Player};
use talv::{board::{Colour, Field, Piece}, game::Game, location::{Coords, File, FileRange, Rank, RankRange}, possible_moves::possible_moves};

const FIELD_SIZE: f32 = 60.;
const TRANSPARENT: Color = Color {
    a: 0.5,
    .. Color::WHITE
};

#[path = "talv_ggez/player.rs"]
mod player;

fn main() {
    let mut b = ContextBuilder::new("talv", "Falch");

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        b = b.resources_dir_name(path);
    }

    let (mut ctx, event_loop) = b
        .window_mode(WindowMode::default().dimensions(8. * FIELD_SIZE, 8. * FIELD_SIZE))
        .window_setup(WindowSetup::default().title("talv"))
        .build()
        .unwrap();

    let arg = env::args().nth(1);
    let arg = arg.as_ref();

    let game_state = GameState::new(&mut ctx, arg.map(|s| s.as_str()), HumanPlayer::default(), Bot1::new()).unwrap();

    ggez::event::run(ctx, event_loop, game_state)
}

struct GameState {
    chess_game: Game,
    board_image: Image,
    pieces_image: Image,
    recent_mesh: Mesh,
    recent_move: Option<(Coords, Coords)>,
    black_player: Box<dyn Player>,
    white_player: Box<dyn Player>,
}

impl GameState {
    fn new<BP: 'static + Player, WP: 'static + Player>(ctx: &mut Context, fen: Option<&str>, white_player: WP, black_player: BP) -> Result<Self, GameError> {
        Ok(GameState {
            board_image: Image::from_path(ctx, "/board.png")?,
            pieces_image: Image::from_path(ctx, "/pieces.png")?,
            recent_mesh: Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(0., 0., FIELD_SIZE, FIELD_SIZE), Color::from_rgba_u32(0xfce2057f))?,
            chess_game: fen.and_then(|s| Game::from_fen(s)).unwrap_or_else(Game::new),
            recent_move: None,
            white_player: Box::new(white_player),
            black_player: Box::new(black_player),
        })
    }

    fn get_player(&self) -> &dyn Player {
        match self.chess_game.side_to_move() {
            Colour::White => &*self.white_player,
            Colour::Black => &*self.black_player,
        }
    }
    fn get_player_mut(&mut self) -> &mut dyn Player {
        match self.chess_game.side_to_move() {
            Colour::White => &mut *self.white_player,
            Colour::Black => &mut *self.black_player,
        }
    }
}

#[inline]
fn xy_to_coords(x: f32, y: f32) -> Option<Coords> {
    let f = File::from_i8((x / FIELD_SIZE) as i8)?;
    let r = Rank::from_i8(7 - (y / FIELD_SIZE) as i8)?;
    Some(Coords::new(f, r))
}

impl EventHandler for GameState {
    fn mouse_button_down_event(
            &mut self,
            _ctx: &mut Context,
            btn: MouseButton,
            x: f32,
            y: f32,
        ) -> Result<(), GameError> {
        if btn != MouseButton::Left {
            return Ok(());
        }
        let Some(coords) = xy_to_coords(x, y) else { return Ok(()) };
        // FIXME
        let bs = self.chess_game.board_state().clone();
        self.get_player_mut().start_interaction(&bs, coords);

        Ok(())
    }
    fn mouse_button_up_event(
            &mut self,
            _ctx: &mut Context,
            btn: MouseButton,
            x: f32,
            y: f32,
        ) -> Result<(), GameError> {
        if btn != MouseButton::Left {
            return Ok(());
        }
        let Some(coords) = xy_to_coords(x, y) else { return Ok(()) };
        // FIXME
        let bs = self.chess_game.board_state().clone();
        self.get_player_mut().end_interaction(&bs, coords);

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let no_moves = possible_moves(self.chess_game.board_state()).is_empty();
        if self.chess_game.is_checked(self.chess_game.side_to_move()) && no_moves {
            println!("Check-mate! {:?} wins.", !self.chess_game.side_to_move());
            ctx.request_quit();
            return Ok(());
        }
        if self.chess_game.draw_claimable() || no_moves {
            println!("Draw");
            ctx.request_quit();
            return Ok(());
        }

        // FIXME
        let bs = self.chess_game.board_state().clone();
        if let Some((from, unto, promotion)) = self.get_player_mut().make_move(&bs) {
            if self.chess_game.make_move(from, unto, promotion) {
                self.recent_move = Some((from, unto));
            }
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, None);
        // Draw board background
        canvas.draw(&self.board_image, DrawParam::new());

        // Draw last move
        if let Some((f, t)) = self.recent_move {
            for coords in [f, t] {
                let (x, y) = coords.i8_tuple();
                let x = x as f32 * FIELD_SIZE;
                let y = (7 - y) as f32 * FIELD_SIZE;

                canvas.draw(&self.recent_mesh, DrawParam::new().dest([x, y]));
            }
        }

        // Draw pieces
        for (r, y) in RankRange::full().rev().zip(0..) {
            for f in FileRange::full() {
                let x = f.i8() as f32 * FIELD_SIZE;
                let y = y as f32 * FIELD_SIZE;
                match self.chess_game.board_state().get(Coords::new(f, r)) {
                    Field::Empty => (),
                    Field::Occupied(c, p) => draw_piece(&mut canvas, &self.pieces_image, x, y, None, c, p),
                }
            } 
        }

        // Draw moving piece
        if let Some(p) = self.get_player().get_interaction() {
            let pos = ctx.mouse.position();
            let x = pos.x - 0.5 * FIELD_SIZE;
            let y = pos.y - 0.5 * FIELD_SIZE;

            draw_piece(&mut canvas, &self.pieces_image, x, y, Some(TRANSPARENT), self.chess_game.side_to_move(), p);
        }

        canvas.finish(ctx)
    }
}

fn draw_piece(canvas: &mut Canvas, pieces_image: &Image, x: f32, y: f32, color: Option<Color>, c: Colour, p: Piece) {
    const SIXTH: f32 = 1./6.;

    let i = match p {
        Piece::Queen => 0,
        Piece::King => 1,
        Piece::Rook => 2,
        Piece::Knight => 3,
        Piece::Bishop => 4,
        Piece::Pawn => 5,
    } as f32 * SIXTH;

    let j = match c {
        Colour::Black => 0.,
        Colour::White => 0.5,
    };

    let mut dp = DrawParam::default()
        .dest([x, y])
        .src(Rect::new(i, j, SIXTH, 0.5));
    if let Some(c) = color {
        dp = dp.color(c);
    }

    canvas.draw(pieces_image, dp);
}