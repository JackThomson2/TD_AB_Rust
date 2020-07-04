pub mod board;
pub mod constants;
pub mod helper;

use board::*;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn main() {
    fastrand::seed(2);
    let game = TDGame::new(false);

    for _ in 0..1000000 {
        let mut gme = game;

        for _i in 0..10 {
            //let pos = fastrand::u8(..8);
            unsafe { gme.step(0) }
        }
    }

    game.render();
}
