use std::io::{stdin, stdout, Write};

use talv::{algebraic::{Move, MoveType}, Game};

fn main() {
    let mut game = Game::new();

    let mut input = String::new();

    loop {
        game.print_board();
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
