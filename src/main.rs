use std::io::{stdin, stdout, Write};

use talv::{Game, location::{Coords}};

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

        let from_to: String = input.chars().filter(|c| c.is_ascii_alphanumeric()).collect();

        if from_to.len() != 4 {
            println!("Wrong length");
            input.clear();
            continue;
        }

        let from = Coords::from_str(&from_to[0..2]);
        let unto = Coords::from_str(&from_to[2..4]);

        match (from, unto) {
            (Some(f), Some(u)) => {
                if game.make_move(f, u) {
                    
                } else {
                    println!("Illegal!");
                }
            }
            _ => println!("Malformed!"),
        }

        input.clear();
    }
}
