#![feature(atomic_min_max)]

pub mod board;
pub mod constants;
pub mod evaluation;
pub mod hash_table;
pub mod helper;
pub mod solver;
pub mod tester;

use board::*;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    fastrand::seed(2);
    let mut game = TDGame::new(false);

    tester::play_game();

    //let mut human = false;
    while !game.game_over() {
        let moving = solver::compute(&game, 22);

        println!("Best move is {} with a score of {}", moving.0, moving.1);
        println!("Board hash is 0b{:0b}", game.hash_me());

        unsafe {
            game.step(moving.0);
        }

        game.render()
    }

    let moving = solver::compute(&game, 8);
    println!("Best move is {} with a score of {}", moving.0, moving.1);

    game.render();
}
