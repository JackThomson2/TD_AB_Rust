pub const ROUND_WEIGHTS: [[u16; 8]; 4] = [
    [2, 2, 2, 2, 2, 2, 2, 2],
    [34, 21, 13, 8, 5, 3, 2, 1],
    [9, 8, 7, 6, 5, 4, 3, 2],
    [64, 49, 36, 25, 16, 9, 4, 1],
];

pub const BIT_SHIFTS: [[u32; 9]; 6] = [
    [0, 1, 2, 3, 0, 0, 0, 0, 0],
    [4, 5, 6, 7, 8, 0, 0, 0, 0],
    [9, 10, 11, 12, 13, 14, 0, 0, 0],
    [15, 16, 17, 18, 19, 20, 21, 0, 0],
    [22, 23, 24, 25, 26, 27, 28, 29, 0],
    [30, 31, 32, 33, 34, 35, 36, 37, 38],
];

pub const RIGHT_PLAYER: u64 = 0b1000;
pub const GAME_OVER: u64 = 0b100;
pub const CURR_ROUND: u64 = 0b11;
pub const ONE: u64 = 0b1;

pub const ROUND_TARGETS: [u32; 4] = [10, 40, 20, 80];