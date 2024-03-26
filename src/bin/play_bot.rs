use std::io::{stdin, stdout, Write};

use talv::{algebraic::Move, board::Colour, bots::bot1, game::Game, possible_moves::possible_moves};

fn main() {
    let mut game;

    let mut input = String::new();

    println!("Input position (FEN) or press enter for new game:");
    stdin().read_line(&mut input).unwrap();
    if input.trim().is_empty() {
        game = Game::new();
    } else {
        game = match Game::from_fen(input.trim()) {
            Some(game) => game,
            None => {
                eprintln!("Invalid FEN string");
                return;
            }
        }
    }
    input.clear();

    loop {
        game.print_game();
        if game.is_checked(game.side_to_move()) {
            println!("Check! ");
            if possible_moves(game.board_state()).is_empty() {
                println!("Mate! {:?} won.", !game.side_to_move());
                break;
            }
        }

        if game.draw_claimable() {
            println!("Draw");
            break;
        }

        match game.side_to_move() {
            Colour::Black => {
                let moves = bot1::get_moves_ranked(game.board_state());
                print!("Ranked moves: ");
                for (e, from, to, p) in &moves {
                    print!("{from}{to}");
                    if let Some(p) = p {
                        print!("={p}");
                    }
                    print!(" ({e:.4}) ");
                }
                println!();
                let (_, from, unto, pr) = moves[0];
                game.make_move(from, unto, pr).then_some(()).unwrap();
            }
            Colour::White => {
                print!("Possible moves: ");
                for (p, from, to, prm) in possible_moves(&game.board_state()) {
                    print!("{p}{from}{to}");
                    if let Some(p) = prm {
                        print!("={p}");
                    }
                    print!(" ");
                }
                println!();
                print!("Move: ");
                stdout().flush().unwrap();

                stdin().read_line(&mut input).unwrap();

                if input.trim().is_empty() {
                    break;
                }

                if let Some(mv) = Move::from_str(input.trim()) {
                    println!("Valid {}", mv);

                    if let Some((f, t, prm)) = game.check_move(mv) {
                        if !game.make_move(f, t, prm) {
                            println!("Illegal!!");
                        }
                    } else {
                        println!("Incorrect {}", mv);
                    }
                }

                input.clear();
            }
        }
    }

    println!(
        "Game was interrupted. Use the following FEN line to continue the game later:\n{}",
        game.display_fen()
    );
}
