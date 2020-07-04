use crate::board::*;
use crate::constants::*;

#[inline]
pub fn get_new_board(_play_holes: bool) -> TDGame {
    let mut board: u64 = fastrand::u64(..);

    board &= !(GAME_OVER
        | CURR_ROUND
        | RIGHT_PLAYER
        | 0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0000);
    TDGame {
        state: board,
        round_scores: [0, 0],
        player_1_score: 0,
        player_2_score: 0,
        next_round: false,
        turn: 0,
    }
}

#[inline]
pub fn get_lever_string(lever: u64) -> &'static str {
    match lever {
        0b00 => "(_ )",
        0b01 => "(0 )",
        0b10 => "( _)",
        0b11 => "( 0)",
        _ => "Uh ob",
    }
}
