use crate::constants::*;
use crate::helper::{get_lever_string, get_new_board};

#[derive(Copy, Clone, Debug)]
pub struct TDGame {
    pub state: u64,
    pub round_scores: [u32; 2],
    pub just_scored: u32,
    pub player_1_score: u32,
    pub player_2_score: u32,
    pub next_round: bool,
    pub turn: u32,
}

impl TDGame {
    pub fn new(holed: bool) -> Self {
        get_new_board(holed)
    }

    pub fn make_avified_board(inp: &str) -> u64 {
        let mut output = 0 << 32;

        let mut cntr = 0;
        for pce in inp.chars() {
            if pce != '\\' {
                output |= RIGHT_PLAYER << (cntr * 2 + 2);
            }
            cntr += 1
        }
        output
    }

    #[inline]
    pub fn game_over(&self) -> bool {
        (self.state & GAME_OVER) == 4
    }

    pub fn render(&self) {
        let mut ctr = 0;
        let mask: u64 = 0b11;

        for cntr in 0..5 {
            let mut print_str = String::new();
            for _ in 0..(2 * (4 - cntr)) {
                print_str.push_str(" ");
            }
            for _ in 0..(4 + cntr) {
                let lever_str =
                    get_lever_string((self.state & (mask << (4 + 2 * ctr))) >> (4 + 2 * ctr));
                print_str.push_str(lever_str);
                ctr += 1;
            }
            println!("{}", print_str);
        }

        println!(
            "Scores: {},{} Player 1 total: {} Player 2 total {}, Round {}, Turn {}",
            self.round_scores[0],
            self.round_scores[1],
            self.player_1_score,
            self.player_2_score,
            1 + (self.state & CURR_ROUND),
            self.turn
        );
    }

    #[inline]
    pub unsafe fn step(&mut self, location: u8) {
        if self.game_over() {
            return;
        }

        let mut coins: [u8; 78] = [0; 78];
        coins[location as usize] = 1;

        let player_num: usize = if self.state & RIGHT_PLAYER != 8 { 0 } else { 1 };
        let row = ROUND_WEIGHTS.get_unchecked((self.state & (CURR_ROUND)) as usize);

        let mut tracked_coins: Vec<(u8, u8)> = vec![get_u8_pos(location as usize)];

        while !tracked_coins.is_empty() {
            let mut next_coins: [u8; 78] = [0; 78];
            let mut searched_map: u64 = 0;

            for position in tracked_coins {
                let pos = 1 << col_to_pos(position);

                if (searched_map & pos) > 0 {
                    continue;
                }

                self.handle_lever_drop(position.0, position.1, &mut coins, &mut next_coins);
                searched_map |= pos;
            }

            for (k, weight) in row.iter().enumerate() {
                let scored_2: u16 = next_coins[76 - k] as u16 * weight;
                let scored: u16 = next_coins[61 + k] as u16 * weight;
                let total = (scored + scored_2) as u32;

                self.just_scored += total;
                self.round_scores[player_num] += total;

                if player_num == 0 {
                    self.player_1_score += total;
                } else {
                    self.player_2_score += total;
                }
            }
            coins = next_coins;

            tracked_coins = coins
                .iter()
                .enumerate()
                .filter(|(pos, val)| **val > 0 && pos < &60)
                .map(|(pos, _val)| get_u8_pos(pos))
                .collect();
        }

        if self.next_round {
            self.next_round = false;
            self.round_scores = [0, 0];
            self.state += 1;
        } else if self.round_scores[player_num as usize]
            >= *ROUND_TARGETS.get_unchecked((self.state & (CURR_ROUND)) as usize)
        {
            self.next_round = true;
        }

        self.turn += 1;
        self.state ^= RIGHT_PLAYER;
    }

    pub unsafe fn hash_me(&self) -> u64 {
        let mut a: u64 = if self.next_round {
            self.state | RIGHT_PLAYER
        } else {
            self.state & !RIGHT_PLAYER
        };

        let mut l: u32 = self.round_scores[0];
        l <<= 16;
        l |= self.round_scores.get_unchecked(1) & 0xffff;
        l <<= 16;
        l |= self.just_scored & 0xffff;
        a = 0xbf58_476d_1ce4_e5b9_u64.wrapping_mul(a ^ (a >> 30));
        a = 0x94d0_49bb_1331_11eb_u64.wrapping_mul(a ^ (a >> 27));
        a = a ^ (a >> 31);
        a ^ l as u64
    }

    #[inline]
    unsafe fn handle_lever_drop(
        &mut self,
        r: u8,
        c: u8,
        coins: &mut [u8; 78],
        next_coins: &mut [u8; 78],
    ) {
        let shift: u32 = *BIT_SHIFTS
            .get_unchecked(r as usize)
            .get_unchecked(c as usize);

        let mut coin: u32 = ((self.state >> (4 + 2 * shift)) & ONE) as u32;
        let head: u32 = ((self.state >> (5 + 2 * shift)) & ONE) as u32;
        let tail: u32 = head ^ 1;

        let head_location = (2 * shift + head) as usize;
        let tail_location = (2 * shift + tail) as usize;

        let head_coin = *coins.get_unchecked(head_location);
        let tail_coin = *coins.get_unchecked(tail_location);

        let mut special_case = false;
        let mut count = 0;

        if head_coin > 0 {
            count = head_coin;
            if coin == 0 {
                coin = 1;
                self.state |= RIGHT_PLAYER << (1 + 2 * shift);
                count -= 1;
                if count > 0 && tail_coin > 0 {
                    special_case = true;
                }
            }
            *coins.get_unchecked_mut(tail_location) += count;
        }

        let tail_coin = *coins.get_unchecked(tail_location);
        // THIS IS FOR IF THE COIN WILL BE FALLING DOWN
        // I.E. It will want to fall to row below
        if tail_coin > 0 {
            let count_2: u8 = tail_coin;
            if coin == 1 && (!special_case || ((count_2 - count) & 1) == 1) {
                self.state &= !(RIGHT_PLAYER << (1 + 2 * shift));
                next_coins[head_location] += 1;
            }

            if !special_case && (count_2 & 1) == 1 {
                self.state ^= RIGHT_PLAYER << (2 + 2 * shift);
            } else if ((count_2 - count) & 1) == 1 {
                self.state ^= RIGHT_PLAYER << (2 + 2 * shift);
            }

            next_coins
                [(2 * BIT_SHIFTS[(r + 1) as usize][(c as u32 + tail) as usize] + head) as usize] +=
                count_2;
        }
    }
}

#[inline]
fn get_u8_pos(pos: usize) -> (u8, u8) {
    let computed = get_row_c(pos);
    (computed.0, (computed.1 / 2) as u8)
}

#[inline]
fn get_row_c(pos: usize) -> (u8, usize) {
    if pos < 8 {
        return (0, pos);
    }
    if pos < 18 {
        let pos = pos - 8;
        return (1, pos);
    }
    if pos < 30 {
        let pos = pos - 18;
        return (2, pos);
    }
    if pos < 44 {
        let pos = pos - 30;
        return (3, pos);
    }
    if pos < 60 {
        let pos = pos - 44;
        return (4, pos);
    }
    let pos = pos - 60;
    return (6, pos + 1);
}

#[inline]
fn col_to_pos(pos: (u8, u8)) -> u8 {
    let row = pos.0;
    let col = pos.1;

    let mut end: u8 = match row {
        0 => 0,
        1 => 8,
        2 => 18,
        3 => 30,
        4 => 44,
        _ => 60,
    };

    end += if row > 4 { col + 1 } else { col };
    end
}
