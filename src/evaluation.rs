use crate::board::*;

#[inline]
pub fn get_scoring(board: &TDGame, left_player: bool) -> i32 {
    let mut score_different = if left_player {
        board.player_1_score as i32 - board.player_2_score as i32
    } else {
        board.player_2_score as i32 - board.player_1_score as i32
    };

    if board.has_parity() {
        score_different -= 2;
    } else {
        score_different += 2;
    }

    score_different
}
