use dashmap::DashMap;

pub const EXACT_FLAG: u8 = 0;
pub const UPPER_FLAG: u8 = 0;
pub const LOWER_FLAG: u8 = 0;

#[derive(Debug, Copy, Clone)]
pub struct TBEntry {
    pub flag: u8,
    pub depth: u8,
    pub score: i32,
}

pub type HashTable = DashMap<u64, TBEntry>;
