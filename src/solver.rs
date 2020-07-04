use crate::board::*;
use crate::evaluation::get_scoring;

use atomic_counter::AtomicCounter;
use rayon::prelude::*;

use std::cmp::{max, min};
use std::sync::Arc;
use std::time::Instant;

const POSSIBE_MOVES: [u8; 8] = [0, 7, 1, 6, 2, 5, 3, 4];

pub fn compute(board: &TDGame, depth: u8) -> (&u8, i32) {
    let now = Instant::now();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));
    let best = POSSIBE_MOVES
        .par_iter()
        .map(|moving| {
            (
                moving,
                make_move(board, depth - 1, *moving, i32::MIN, i32::MAX, &cntr),
            )
        })
        .max_by(|x, y| x.1.cmp(&y.1))
        .unwrap();

    println!(
        "Took {:?}, Searched {} positions",
        now.elapsed(),
        cntr.get()
    );

    best
}

fn make_move(
    board: &TDGame,
    depth: u8,
    move_to: u8,
    alpha: i32,
    beta: i32,
    cntr: &atomic_counter::RelaxedCounter,
) -> i32 {
    let mut copied = *board;
    cntr.inc();

    let mut alpha = alpha;

    if copied.game_over() {
        return -111;
    }

    let is_left = copied.is_left_player();
    let _score = unsafe { copied.step(move_to) };

    if copied.game_over() {
        return i32::MAX;
    }

    if depth == 0 {
        return get_scoring(&copied, is_left);
    }

    let mut best_score = i32::MIN;

    for mv in POSSIBE_MOVES.iter() {
        let res = if depth > 5 {
            rayon::scope(|_| -make_move(&copied, depth - 1, *mv, -beta, -alpha, cntr))
        } else {
            -make_move(&copied, depth - 1, *mv, -beta, -alpha, cntr)
        };

        if res > best_score {
            best_score = res;
        }

        alpha = max(alpha, best_score);
        if alpha >= beta {
            break;
        }
    }

    return best_score;
}
