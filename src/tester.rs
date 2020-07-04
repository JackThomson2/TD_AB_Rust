use crate::board::*;
use crate::solver::compute;

use std::io;

pub fn play_game() {
    let mut board = TDGame::new(true);
    let mut human = false;

    while !board.game_over() {
        board.render();
        let moving = if !human {
            compute(&board, 17).0
        } else {
            read_input()
        };

        human = !human;

        println!("Moving to {}", moving);

        unsafe {
            board.step(moving);
        }
    }
}

fn read_input() -> u8 {
    let mut input = String::new();
    println!("Enter move:  ");
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                let bytes: u8 = input.trim().parse().unwrap();
                return bytes - 1;
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
