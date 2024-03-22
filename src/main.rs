use std::io::{stdin, stdout, Write};

use talv::{algebraic::Move, Game};

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
        }
        if game.is_checked(!game.side_to_move()) {
            println!("Illegal check! ");
        }
        print!("Move: ");
        stdout().flush().unwrap();

        stdin().read_line(&mut input).unwrap();

        if input.trim().is_empty() {
            break;
        }

        let mv = Move::from_str(input.trim());

        if let Some(mv) = mv {
            println!("Valid {}", mv);

            if let Some((f, t)) = game.check_move(mv) {
                if game.make_move(f, t) {
                    if let Some(promotion) = mv.promotion() {
                        if !game.promote(promotion) {
                            println!("Illegal promotion to {}, ignored", promotion);
                        }
                    }
                } else {
                    println!("Illegal!!");
                }
            } else {
                println!("Incorrect {}", mv);
            }
        }

        input.clear();
    }
}
