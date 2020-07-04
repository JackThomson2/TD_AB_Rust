pub mod board;
pub mod constants;
pub mod evaluation;
pub mod helper;
pub mod solver;

use board::*;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    fastrand::seed(2);
    let mut game = TDGame::new(false);

    /* for _ in 0..1000000 {
        let mut gme = game;

        for _i in 0..10 {
            //let pos = fastrand::u8(..8);
            unsafe {
                gme.step(0);
            }
        }
    }*/

    //let mut human = false;
    while !game.game_over() {
        let moving = solver::compute(&game, 12);

        println!("Best move is {} with a score of {}", moving.0, moving.1);
        println!("Board hash is {}", game.hash_me());

        unsafe {
            game.step(*moving.0);
        }

        game.render()
    }

    let moving = solver::compute(&game, 8);
    println!("Best move is {} with a score of {}", moving.0, moving.1);

    game.render();
}
