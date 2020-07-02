use std::collections::HashSet;

const ROUND_WEIGHTS: [[u16; 8]; 4] = [
    [2, 2, 2, 2, 2, 2, 2, 2],
    [34, 21, 13, 8, 5, 3, 2, 1],
    [9, 8, 7, 6, 5, 4, 3, 2],
    [64, 49, 36, 25, 16, 9, 4, 1],
];

const BIT_SHIFTS: [[u32; 9]; 6] = [
    [0, 1, 2, 3, 0, 0, 0, 0, 0],
    [4, 5, 6, 7, 8, 0, 0, 0, 0],
    [9, 10, 11, 12, 13, 14, 0, 0, 0],
    [15, 16, 17, 18, 19, 20, 21, 0, 0],
    [22, 23, 24, 25, 26, 27, 28, 29, 0],
    [30, 31, 32, 33, 34, 35, 36, 37, 38],
];

const ROW_OFFSET: [usize; 6] = [9, 11, 13, 15, 17, 19];

const ROW_RANGE: [[u8; 2]; 5] = [[0, 8], [8, 18], [18, 30], [30, 44], [44, 60]];

const RIGHT_PLAYER: u64 = 0b1000;
const GAME_OVER: u64 = 0b100;
const CURR_ROUND: u64 = 0b11;
const ROW_END: u8 = 0b111;
const ONE: u64 = 0b1;
const ROUND_TARGETS: [u32; 4] = [10, 40, 20, 80];

const WHITE_PLAYER: u8 = 1;
const DRAW: u8 = 2;
const BLACK_PLAYER: u8 = 3;

#[derive(Clone)]
pub struct TDGame {
    pub state: u64,
    round_scores: [u32; 2],
    just_scored: u32,
    pub player_1_score: u32,
    pub player_2_score: u32,
    next_round: bool,
    turn: u32,
    pub holes: Vec<u8>,

    play_holes: bool,
}

impl TDGame {
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

        let head_location: u32 = 2 * shift + head;
        let tail_location: u32 = 2 * shift + tail;

        let head_coin = *coins.get_unchecked(head_location as usize);
        let tail_coin = *coins.get_unchecked(tail_location as usize);

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
            coins[tail_location as usize] += count;
        }

        let tail_coin = coins[tail_location as usize];
        // THIS IS FOR IF THE COIN WILL BE FALLING DOWN
        // I.E. It will want to fall to row below
        if tail_coin > 0 {
            // Check that we location is a hole is so we'll teleport
            if self.play_holes && self.holes.contains(&(tail_location as u8)) {
                let count_2: u8 = tail_coin;

                if coin == 1 && (!special_case || ((count_2 - count) & 1) == 1) {
                    self.state &= !(RIGHT_PLAYER << (1 + 2 * shift));
                }

                // THIS SECION IS FOR SWAPPING LEVER THE COIN JUST FELL THROUGH
                if !special_case {
                    if (count_2 & 1) == 1 {
                        self.state ^= RIGHT_PLAYER << (2 + 2 * shift);
                    }
                } else if ((count_2 - count) & 1) == 1 {
                    self.state ^= RIGHT_PLAYER << (2 + 2 * shift);
                }

                // HERE WE ARE FINDING THE RANDOM HOLE TO TELEPORT TO
                let mut hole_pos: usize = fastrand::usize(..self.holes.len());

                // Prevent a coin from teleporting the same location
                while self.holes[hole_pos] == tail_location as u8 {
                    hole_pos = fastrand::usize(..self.holes.len())
                }

                // The position on our arrary
                let chosen_pos = self.holes[hole_pos] as usize;
                // Get the row this piece is on
                let (row, col) = get_row_c(chosen_pos);

                // Attempt the map the position from the holes to the coins
                let location = chosen_pos + ROW_OFFSET[row];

                // Update the new position on the next array to our coins location
                next_coins[location] += tail_coin;

                // Handle flipping another falling coin
                if coin == 1 && (!special_case || ((count_2 - count) & 1) == 1) {
                    self.state &= !(RIGHT_PLAYER << (1 + 2 * shift));
                    next_coins[head_location as usize] += 1;
                }

                /*println!(
                    "Teleported coin from point {},{} to point {},{}",
                    r, c, row, col
                );*/
                return;
            }

            let count_2: u8 = tail_coin;
            if coin == 1 && (!special_case || ((count_2 - count) & 1) == 1) {
                self.state &= !(RIGHT_PLAYER << (1 + 2 * shift));
                next_coins[head_location as usize] += 1;
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

impl TDGame {
    fn new(holed: bool) -> Self {
        get_new_board(holed)
    }

    fn copy(copying: &TDGame) -> TDGame {
        copying.clone()
    }

    fn make_avified_board(inp: &str) -> u64 {
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

    pub fn make_holey_avified_board(inp: &str) -> (u64, Vec<u8>) {
        let mut output = 0 << 32;
        let mut holes: Vec<u8> = Vec::new();

        let mut cntr = 0;
        for pce in inp.chars() {
            if pce == '/' {
                output |= RIGHT_PLAYER << (cntr * 2 + 2);
            } else if pce == '0' {
                output |= RIGHT_PLAYER << (cntr * 2 + 2);
                holes.push(cntr * 2);
            } else if pce == '1' {
                output |= RIGHT_PLAYER << (cntr * 2 + 2);
                holes.push((cntr * 2) + 1);
            } else if pce == '2' {
                holes.push(cntr * 2);
            } else if pce == '3' {
                holes.push((cntr * 2) + 1);
            }
            cntr += 1
        }
        (output, holes)
    }

    fn read_board(inp: &str) -> u64 {
        let mut output = 0 << 32;
        let mut cntr = 0;

        for pce in inp.chars() {
            if pce != '1' {
                output |= RIGHT_PLAYER << (cntr * 2 + 2);
            }
            cntr += 1;
        }
        output
    }

    fn game_over(&self) -> bool {
        (self.state & GAME_OVER) == 4
    }

    pub fn done(&self) -> bool {
        self.game_over()
    }

    fn white_to_move(&self) -> bool {
        (self.state & RIGHT_PLAYER) != 8
    }

    fn winner(&self) -> Option<u8> {
        if !self.game_over() {
            None
        } else if self.player_1_score > self.player_2_score {
            Some(WHITE_PLAYER)
        } else if self.player_2_score > self.player_1_score {
            Some(BLACK_PLAYER)
        } else {
            Some(DRAW)
        }
    }

    fn white_won(&self) -> bool {
        if let Some(winner) = self.winner() {
            return winner == WHITE_PLAYER;
        }
        false
    }

    fn player_turn(&self) -> u8 {
        if self.turn % 2 == 0 {
            WHITE_PLAYER
        } else {
            BLACK_PLAYER
        }
    }

    fn copy_me(&self) -> TDGame {
        self.clone()
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
            let row = ROW_RANGE[cntr];
            let range = draw_hole_range(&self.holes, (row[0]..row[1]).collect());
            let mut print_str = String::new();
            for _ in 0..(2 * (4 - cntr)) {
                print_str.push_str(" ");
            }
            print_str.push_str(&range);

            println!("{}", print_str);
        }

        println!(
            "Scores : {}, {} Player 1 total: {} Player 2 total {}, Round {}, Turn {}",
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
        if (self.state & GAME_OVER) == 4 {
            return;
        }

        let mut searched_map: HashSet<(u8, u8)> = HashSet::new();
        let mut coins: [u8; 78] = [0; 78];
        coins[location as usize] = 1;

        let player_num: usize = if self.state & RIGHT_PLAYER != 8 { 0 } else { 1 };
        let row = ROUND_WEIGHTS.get_unchecked((self.state & (CURR_ROUND)) as usize);

        let mut tracked_coins: Vec<(u8, u8)> = coins
            .iter()
            .enumerate()
            .filter(|(pos, val)| **val > 0 && pos < &60)
            .map(|(pos, _val)| get_row_c(pos))
            .map(|(r, c)| (r as u8, (c / 2) as u8))
            .collect();

        while !tracked_coins.is_empty() {
            let mut next_coins: [u8; 78] = [0; 78];
            searched_map.clear();

            for position in tracked_coins {
                if searched_map.contains(&position) {
                    continue;
                }
                self.handle_lever_drop(position.0, position.1, &mut coins, &mut next_coins);
                searched_map.insert(position);
            }

            /*coins
                .iter()
                .enumerate()
                .filter(|(_pos, val)| **val > 0)
                .for_each(|(pos, _val)| println!("Coin at {}", pos));
            next_coins
                .iter()
                .enumerate()
                .filter(|(_pos, val)| **val > 0)
                .for_each(|(pos, _val)| println!("Next coin at {}", pos));
            println!("Holes {:?}", &self.holes[..]);
            draw_coins(&coins[..coins.len()], "Coins");
            draw_coins(&next_coins, "Next Coins");*/

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
                .map(|(pos, _val)| get_row_c(pos))
                .map(|(r, c)| (r as u8, (c / 2) as u8))
                .collect();
        }

        if self.next_round {
            self.next_round = false;
            self.round_scores = [0, 0];
            self.state += 1;
            if self.play_holes {
                self.holes = gen_holes_for_rnd((self.state & (CURR_ROUND)) as usize + 1);
            }
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
}

fn draw_coins(coins: &[u8], name: &str) {
    println!("#####################################");
    println!("Drop for {} \n\n ", name);
    print!("            ");
    for (pos, coin) in coins.iter().enumerate() {
        if pos == 8 {
            println!();
            print!("          ");
        } else if pos == 18 {
            println!();
            print!("        ");
        } else if pos == 30 {
            println!();
            print!("      ");
        } else if pos == 44 {
            println!();
            print!("    ");
        } else if pos == 60 {
            println!();
            print!("  ");
        }
        print!("{},", coin);
    }
    println!();
    println!("\n\n#####################################");
}

#[inline]
fn get_row_c(pos: usize) -> (usize, usize) {
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
fn draw_hole_range(row: &[u8], hole_info: Vec<u8>) -> String {
    let mut built_str = String::new();
    for (pos, pce) in hole_info.iter().enumerate() {
        built_str.push_str(&format!("|{}", if row.contains(pce) { "X" } else { " " }));
    }
    built_str.push_str("|");
    built_str
}

#[inline]
fn get_lever_id(lever: u64) -> i32 {
    match lever {
        0b00 => -1,
        0b01 => -2,
        0b10 => 1,
        0b11 => 2,
        _ => -99,
    }
}

#[inline]
fn gen_holes_for_rnd(round: usize) -> Vec<u8> {
    let mut holes = Vec::with_capacity(2 + round);

    for _ in 0..2 + round {
        let mut pos = fastrand::u8(..=60);

        while holes.contains(&pos) {
            pos = fastrand::u8(..=60);
        }
        holes.push(pos);
    }

    holes
}

#[inline]
pub fn get_new_board(play_holes: bool) -> TDGame {
    let mut board: u64 = fastrand::u64(..);
    let mut holes = if play_holes {
        gen_holes_for_rnd(1)
    } else {
        vec![]
    };

    board &= !(GAME_OVER
        | CURR_ROUND
        | RIGHT_PLAYER
        | 0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0000);
    TDGame {
        state: board,
        holes,
        round_scores: [0, 0],
        just_scored: 0,
        player_1_score: 0,
        player_2_score: 0,
        next_round: false,
        turn: 0,
        play_holes,
    }
}

#[inline]
fn get_lever_string(lever: u64) -> &'static str {
    match lever {
        0b00 => "(_ )",
        0b01 => "(0 )",
        0b10 => "( _)",
        0b11 => "( 0)",
        _ => "Uh ob",
    }
}
