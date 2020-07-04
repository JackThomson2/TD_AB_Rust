use crate::board::*;
use crate::evaluation::get_scoring;
use crate::hash_table::*;

use std::sync::atomic::{AtomicI32, Ordering};

use atomic_counter::AtomicCounter;
use rayon::prelude::*;

use std::cmp::{max, min};
use std::sync::Arc;
use std::time::Instant;

const POSSIBE_MOVES: [u8; 8] = [0, 7, 1, 6, 2, 5, 3, 4];

pub fn compute(board: &TDGame, depth: u8) -> (u8, i32) {
    let hash_table: HashTable = dashmap::DashMap::with_capacity(40_000_000);

    let now = Instant::now();
    let cntr = Arc::new(atomic_counter::RelaxedCounter::new(0));
    let hash_hits = Arc::new(atomic_counter::RelaxedCounter::new(0));

    let mut best_score = i32::MIN;
    let mut best_move: u8 = 0;

    let mut alpha = -999999;
    let beta = 999999;

    // Synchronous at the top performs better?
    for moves in POSSIBE_MOVES.iter() {
        let mut brd_copy = *board;
        unsafe { brd_copy.step(*moves) };

        let score = -nega_max(
            &brd_copy,
            depth - 1,
            beta * -1,
            alpha * -1,
            &cntr,
            &hash_hits,
            &hash_table,
        );

        if score > best_score {
            best_score = score;
            best_move = *moves;
            alpha = score;
        }
    }

    println!(
        "Took {:?}, Move is {} with score {}.\n Searched {} positions, with {} hashhits, table size is {}",
        now.elapsed(),
        best_move,
        best_score,
        cntr.get(),
        hash_hits.get(),
        hash_table.len()
    );

    (best_move, best_score)
}

#[inline]
fn nega_max(
    board: &TDGame,
    depth: u8,
    alpha: i32,
    beta: i32,
    cntr: &atomic_counter::RelaxedCounter,
    hash_hits: &atomic_counter::RelaxedCounter,
    hash_table: &HashTable,
) -> i32 {
    if depth == 0 {
        cntr.inc();
        let is_left = board.is_left_player();
        return get_scoring(board, is_left);
    }

    let alpha_orig = alpha;
    let mut new_alpha = alpha;
    let mut beta = beta;

    let hash = board.hash_me();

    if let Some(found) = hash_table.get(&hash) {
        if found.depth >= depth {
            hash_hits.inc();
            if found.flag == EXACT_FLAG {
                return found.score;
            } else if found.flag == LOWER_FLAG {
                new_alpha = max(new_alpha, found.score);
            } else if found.flag == UPPER_FLAG {
                beta = min(beta, found.score);
            }

            if new_alpha >= beta {
                return found.score;
            }
        }
    }

    let alpha = AtomicI32::new(new_alpha);
    let best_move = AtomicI32::new(-9999);

    let loop_fun = |moving: &u8| {
        if beta < alpha.load(Ordering::SeqCst) {
            return;
        }
        let mut new_board = *board;
        unsafe { new_board.step(*moving) };
        let score = -nega_max(
            &new_board,
            depth - 1,
            -beta,
            -alpha.load(Ordering::SeqCst),
            cntr,
            hash_hits,
            hash_table,
        );

        best_move.fetch_max(score, Ordering::Relaxed);
        alpha.fetch_max(score, Ordering::Relaxed);
    };

    if depth > 4 {
        POSSIBE_MOVES.par_iter().for_each(loop_fun);
    } else {
        POSSIBE_MOVES.iter().for_each(loop_fun);
    }
    let final_score = best_move.load(Ordering::SeqCst);

    let entry = TBEntry {
        depth,
        score: final_score,
        flag: if final_score <= alpha_orig {
            UPPER_FLAG
        } else if final_score >= beta {
            LOWER_FLAG
        } else {
            EXACT_FLAG
        },
    };

    hash_table.insert(hash, entry);

    final_score
}
